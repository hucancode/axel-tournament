use axel_judge::{spawn_judge_server, JudgeConfig};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tokio::time::sleep;

const RPS_SERVER_CODE: &str = include_str!("../../games/rock_paper_scissor/server.rs");
const RPS_PLAYER_ROCK: &str = include_str!("../../games/rock_paper_scissor/client_rock.rs");
const RPS_PLAYER_ALT: &str = include_str!("../../games/rock_paper_scissor/client_cycle.rs");

/// E2E test for judge service:
/// 1. Create a rock_paper_scissor game with server code
/// 2. Create two submissions with player code
/// 3. Create a match in "pending" status
/// 4. Wait for judge to execute the match
/// 5. Verify match is completed with scores
#[tokio::test]
async fn judge_executes_rock_paper_scissor_match() {
    // Use production config from environment
    let judge_config = JudgeConfig::from_env();
    tracing_subscriber::fmt::init();
    let judge_handle = spawn_judge_server(judge_config.clone());
    // Give the LIVE query a moment to subscribe
    sleep(Duration::from_secs(1)).await;

    // Connect to same database as judge
    let db_url = judge_config.db_url.trim_start_matches("ws://");
    let db: Surreal<Client> = Surreal::new::<Ws>(db_url)
        .await
        .expect("Failed to connect to database");

    db.signin(Root {
        username: &judge_config.db_user,
        password: &judge_config.db_pass,
    })
    .await
    .expect("Failed to sign in");

    db.use_ns(&judge_config.db_ns)
        .use_db(&judge_config.db_name)
        .await
        .expect("Failed to use namespace/database");

    println!("Connected to test database");

    // Step 1: Create a game with rock_paper_scissor server code
    let game_code = RPS_SERVER_CODE;

    // Use raw SQL query to insert game record
    let owner_thing: Thing = "user:judge_owner"
        .parse()
        .expect("Failed to parse owner id");
    let mut result = db
        .query(
            "CREATE game SET name = $name, description = $description, supported_languages = $supported_languages, owner_id = $owner_id, game_code = $code, game_language = $lang, rounds_per_match = $rounds_per_match, repetitions = $repetitions, timeout_ms = $timeout_ms, cpu_limit = $cpu_limit, turn_timeout_ms = $turn_timeout_ms, memory_limit_mb = $memory_limit_mb, created_at = time::now(), updated_at = time::now() RETURN id",
        )
        .bind(("name", "Rock Paper Scissor"))
        .bind(("description", "RPS game for judge e2e test"))
        .bind(("supported_languages", json!(["rust"])))
        .bind(("owner_id", owner_thing))
        .bind(("code", game_code))
        .bind(("lang", "rust"))
        .bind(("rounds_per_match", 100))
        .bind(("repetitions", 1))
        .bind(("timeout_ms", 2000))
        .bind(("cpu_limit", 1.0))
        .bind(("turn_timeout_ms", 2000))
        .bind(("memory_limit_mb", 64))
        .await
        .expect("Failed to create game");

    #[derive(Deserialize)]
    struct GameId {
        id: Thing,
    }

    let game: Vec<GameId> = result.take(0).expect("Failed to get game from result");
    let game_id = game.first().expect("Game should be created").id.to_string();
    println!("Created game: {}", game_id);

    // Verify game can be fetched
    let game_thing: Thing = game_id.parse().expect("Failed to parse game ID");
    let mut verify = db
        .query("SELECT * FROM $game_id")
        .bind(("game_id", game_thing.clone()))
        .await
        .expect("Failed to query game");

    #[derive(Deserialize, Debug)]
    struct GameVerify {
        game_code: String,
        game_language: String,
    }

    let verified_games: Vec<GameVerify> =
        verify.take(0).expect("Failed to parse game verification");
    let verified_game = verified_games
        .first()
        .expect("Game should exist after creation");
    println!("Verified game exists: {:?}", verified_game);
    assert!(!verified_game.game_code.is_empty());
    assert!(!verified_game.game_language.is_empty());

    // Step 2: Create a tournament for submissions
    let mut result = db
        .query(
            "CREATE tournament SET
                game_id = $game_id,
                name = $name,
                description = $description,
                status = $status,
                min_players = $min_players,
                max_players = $max_players,
                current_players = $current_players,
                match_generation_type = $match_generation_type,
                created_at = time::now(),
                updated_at = time::now()
             RETURN id",
        )
        .bind(("game_id", game_thing.clone()))
        .bind(("name", "Judge E2E Tournament"))
        .bind(("description", "Tournament for judge e2e test"))
        .bind(("status", "scheduled"))
        .bind(("min_players", 2))
        .bind(("max_players", 2))
        .bind(("current_players", 0))
        .bind(("match_generation_type", "all_vs_all"))
        .await
        .expect("Failed to create tournament");

    #[derive(Deserialize)]
    struct TournamentId {
        id: Thing,
    }

    let tournament: Vec<TournamentId> = result
        .take(0)
        .expect("Failed to get tournament from result");
    let tournament_id = tournament
        .first()
        .expect("Tournament should be created")
        .id
        .to_string();
    println!("Created tournament: {}", tournament_id);
    let tournament_thing: Thing = tournament_id
        .parse()
        .expect("Failed to parse tournament ID");

    // Step 3: Create two submissions with player code
    let player1_code = RPS_PLAYER_ROCK;
    let player2_code = RPS_PLAYER_ALT;
    let user1_thing: Thing = "user:test_user1"
        .parse()
        .expect("Failed to parse user 1 ID");
    let user2_thing: Thing = "user:test_user2"
        .parse()
        .expect("Failed to parse user 2 ID");

    let mut result = db
        .query("CREATE submission SET game_id = $game_id, tournament_id = $tournament_id, user_id = $user_id, code = $code, language = $lang, status = 'active', created_at = time::now() RETURN id")
        .bind(("game_id", game_thing.clone()))
        .bind(("tournament_id", tournament_thing.clone()))
        .bind(("user_id", user1_thing))
        .bind(("code", player1_code))
        .bind(("lang", "rust"))
        .await
        .expect("Failed to create submission 1");

    #[derive(Deserialize)]
    struct SubmissionId {
        id: Thing,
    }

    let submission1: Vec<SubmissionId> = result
        .take(0)
        .expect("Failed to get submission1 from result");
    let submission1_id = submission1
        .first()
        .expect("Submission 1 should be created")
        .id
        .to_string();
    println!("Created submission 1: {}", submission1_id);
    let submission1_thing: Thing = submission1_id
        .parse()
        .expect("Failed to parse submission 1 ID");

    let mut result = db
        .query("CREATE submission SET game_id = $game_id, tournament_id = $tournament_id, user_id = $user_id, code = $code, language = $lang, status = 'active', created_at = time::now() RETURN id")
        .bind(("game_id", game_thing.clone()))
        .bind(("tournament_id", tournament_thing.clone()))
        .bind(("user_id", user2_thing))
        .bind(("code", player2_code))
        .bind(("lang", "rust"))
        .await
        .expect("Failed to create submission 2");

    let submission2: Vec<SubmissionId> = result
        .take(0)
        .expect("Failed to get submission2 from result");
    let submission2_id = submission2
        .first()
        .expect("Submission 2 should be created")
        .id
        .to_string();
    println!("Created submission 2: {}", submission2_id);
    let submission2_thing: Thing = submission2_id
        .parse()
        .expect("Failed to parse submission 2 ID");

    // Step 4: Create a match in "pending" status
    let mut result = db
        .query("CREATE match SET game_id = $game_id, tournament_id = $tournament_id, status = 'pending', participants = $participants, created_at = time::now(), updated_at = time::now() RETURN id")
        .bind(("game_id", game_thing.clone()))
        .bind(("tournament_id", tournament_thing.clone()))
        .bind(("participants", json!([
            {
                "submission_id": submission1_thing
            },
            {
                "submission_id": submission2_thing
            }
        ])))
        .await
        .expect("Failed to create match");

    #[derive(Deserialize)]
    struct MatchId {
        id: Thing,
    }

    let match_record: Vec<MatchId> = result.take(0).expect("Failed to get match from result");
    let match_id = match_record
        .first()
        .expect("Match should be created")
        .id
        .to_string();
    println!("Created match: {} (status: pending)", match_id);

    // Step 4: Wait for judge to execute the match (max 60 seconds)
    println!("Waiting for judge to execute the match...");

    let match_thing: Thing = match_id.parse().expect("Failed to parse match ID as Thing");

    for attempt in 1..=60 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        #[derive(Deserialize, Debug)]
        struct MatchStatus {
            status: String,
            #[serde(default)]
            participants: Vec<MatchParticipant>,
            #[serde(default)]
            started_at: Option<String>,
            #[serde(default)]
            completed_at: Option<String>,
            #[serde(default)]
            metadata: Option<serde_json::Value>,
        }

        #[derive(Deserialize, Debug)]
        struct MatchParticipant {
            #[serde(default)]
            score: Option<f64>,
        }

        let mut result = db
            .query("SELECT status, participants, started_at, completed_at, metadata FROM $match_id")
            .bind(("match_id", match_thing.clone()))
            .await
            .expect("Failed to fetch match");

        let matches: Vec<MatchStatus> = result.take(0).expect("Failed to get match results");

        if let Some(match_data) = matches.first() {
            println!(
                "[Attempt {}/60] Match status: {}",
                attempt, match_data.status
            );

            if match_data.status == "failed" {
                println!("Match failed. Metadata: {:?}", match_data.metadata);
                let _ = judge_handle.shutdown().await;
                panic!("Match execution failed");
            } else if match_data.status == "completed" {
                println!("Match completed!");
                // Step 5: Verify the match has scores
                assert_eq!(
                    match_data.participants.len(),
                    2,
                    "Should have 2 participants"
                );
                println!("=== Match Results ===");
                let mut scores: Vec<f64> = Vec::new();
                for participant in match_data.participants.iter() {
                    let score = participant.score.expect("Score should be present");
                    scores.push(score);
                }
                // Ensure both players produced valid (non-error) scores
                assert!(
                    scores.iter().all(|score| *score > 0.0),
                    "Both players should have a positive score"
                );
                // RPS game should finish nearly tied
                let score_diff = (scores[0] - scores[1]).abs();
                assert!(
                    score_diff <= 1.0,
                    "Scores should not differ by more than 1 point, diff: {}",
                    score_diff
                );
                // Verify metadata exists
                assert!(
                    match_data.started_at.is_some(),
                    "Match should have started_at timestamp"
                );
                assert!(
                    match_data.completed_at.is_some(),
                    "Match should have completed_at timestamp"
                );
                let _ = judge_handle.shutdown().await;
                return;
            }
        }
    }

    let _ = judge_handle.shutdown().await;
    panic!("Judge did not execute the match within 60 seconds");
}
