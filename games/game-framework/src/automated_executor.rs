use crate::docker_player::DockerPlayer;
use crate::game_trait::{GameConfig, GameLogic};
use tokio::time::Duration;
use tracing::warn;

/// Execute an automated match between two bots
/// Returns score string in format: "score1 score2" or error code like "TLE 0"
pub async fn execute_automated_match<G: GameLogic>(
    player1_binary: &str,
    player2_binary: &str,
    config: &GameConfig,
) -> String {
    // Spawn Docker containers for both players
    let mut player1 = match DockerPlayer::new(
        player1_binary,
        config.memory_limit_mb,
        config.cpu_limit,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to create player 1 container: {}", e);
            return "RE 0".to_string();
        }
    };

    let mut player2 = match DockerPlayer::new(
        player2_binary,
        config.memory_limit_mb,
        config.cpu_limit,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to create player 2 container: {}", e);
            player1.cleanup().await;
            return "0 RE".to_string();
        }
    };

    let timeout_duration = Duration::from_millis(config.turn_timeout_ms);
    let mut game_state = G::new_game();
    let mut round = 0;

    // Main game loop
    while !G::is_game_over(&game_state) && round < config.num_rounds {
        // Send game state to both players
        let state_p1 = G::encode_state_for_player(&game_state, 0);
        let state_p2 = G::encode_state_for_player(&game_state, 1);

        if player1.send(&state_p1).await.is_err() {
            player1.cleanup().await;
            player2.cleanup().await;
            return format!("RE {}", G::get_scores(&game_state)[1]);
        }

        if player2.send(&state_p2).await.is_err() {
            player1.cleanup().await;
            player2.cleanup().await;
            return format!("{} RE", G::get_scores(&game_state)[0]);
        }

        // Read moves from both players with timeout
        let response1 = match player1.read_with_timeout(timeout_duration).await {
            Ok(r) => r,
            Err(e) => {
                player1.cleanup().await;
                player2.cleanup().await;
                let error_code = if e.to_string().contains("TLE") {
                    "TLE"
                } else {
                    "RE"
                };
                return format!("{} {}", error_code, G::get_scores(&game_state)[1]);
            }
        };

        let response2 = match player2.read_with_timeout(timeout_duration).await {
            Ok(r) => r,
            Err(e) => {
                player1.cleanup().await;
                player2.cleanup().await;
                let error_code = if e.to_string().contains("TLE") {
                    "TLE"
                } else {
                    "RE"
                };
                return format!("{} {}", G::get_scores(&game_state)[0], error_code);
            }
        };

        // Parse moves
        let move1 = match G::parse_move(&response1) {
            Ok(m) => m,
            Err(_) => {
                player1.cleanup().await;
                player2.cleanup().await;
                return format!("WA {}", G::get_scores(&game_state)[1]);
            }
        };

        let move2 = match G::parse_move(&response2) {
            Ok(m) => m,
            Err(_) => {
                player1.cleanup().await;
                player2.cleanup().await;
                return format!("{} WA", G::get_scores(&game_state)[0]);
            }
        };

        // Apply moves to game state
        if let Err(_) = G::make_move(&mut game_state, 0, &move1) {
            player1.cleanup().await;
            player2.cleanup().await;
            return format!("WA {}", G::get_scores(&game_state)[1]);
        }

        if let Err(_) = G::make_move(&mut game_state, 1, &move2) {
            player1.cleanup().await;
            player2.cleanup().await;
            return format!("{} WA", G::get_scores(&game_state)[0]);
        }

        round += 1;
    }

    // Cleanup containers
    player1.cleanup().await;
    player2.cleanup().await;

    // Return final scores
    let scores = G::get_scores(&game_state);
    format!("{} {}", scores[0], scores[1])
}
