mod common;

use axel_tournament::{
    config::DatabaseConfig,
    db,
    models::ProgrammingLanguage,
    services::{auth::AuthService, game, leaderboard, tournament, user},
};

async fn setup_test_db() -> axel_tournament::db::Database {
    let namespace = common::unique_name("test_leaderboard_");
    let config = DatabaseConfig {
        url: "ws://127.0.0.1:8000".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: namespace.clone(),
        database: namespace,
    };
    db::connect(&config)
        .await
        .expect("Failed to connect to test database")
}

const DEFAULT_GAME_CODE: &str = "fn main() {}";
const DEFAULT_ROUNDS_PER_MATCH: u32 = 3;
const DEFAULT_REPETITIONS: u32 = 1;
const DEFAULT_TIMEOUT_SECONDS: u32 = 120;
const DEFAULT_CPU_LIMIT: &str = "1.0";
const DEFAULT_MEMORY_LIMIT: &str = "512m";

fn default_owner_id() -> String {
    "user:owner".to_string()
}

#[tokio::test]
async fn test_leaderboard_ordering_and_limit() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("secret".to_string(), 3600);
    let game = game::create_game(
        &db,
        common::unique_name("Leaderboard Game "),
        "Desc".to_string(),
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_SECONDS,
        DEFAULT_CPU_LIMIT.to_string(),
        DEFAULT_MEMORY_LIMIT.to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap();
    let tournament = tournament::create_tournament(
        &db,
        game_id,
        common::unique_name("Leaderboard Tournament "),
        "Test tournament".to_string(),
        2,
        32,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap();
    // Create two players and join tournament
    let mut participants = Vec::new();
    for score in [120.0, 60.0] {
        let password_hash = auth_service.hash_password("password123").unwrap();
        let player = user::create_user(
            &db,
            format!("{}@test.com", common::unique_name("leaderboard_user")),
            common::unique_name("player"),
            Some(password_hash),
            "US".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        let player_id = player.id.unwrap();
        let participant = tournament::join_tournament(&db, tournament_id.clone(), player_id)
            .await
            .unwrap();
        participants.push((participant.id.unwrap(), score));
    }
    // Assign scores
    for (participant_id, score) in participants {
        db.query("UPDATE $id SET score = $score, rank = 1")
            .bind(("id", participant_id.clone()))
            .bind(("score", score))
            .await
            .unwrap();
    }
    let leaderboard_entries =
        leaderboard::get_leaderboard(&db, 1, Some(tournament_id.clone()), None)
            .await
            .unwrap();
    assert_eq!(leaderboard_entries.len(), 1);
    assert!(leaderboard_entries[0].score >= 100.0);
    assert_eq!(leaderboard_entries[0].rank, 1);
}
