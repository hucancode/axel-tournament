mod common;

use axel_tournament::{
    db,
    services::{auth::AuthService, leaderboard, tournament, user},
};

async fn setup_test_db() -> axel_tournament::db::Database {
    let config = axel_tournament::config::Config::from_env()
        .expect("Failed to load config from environment");
    db::connect(&config.database)
        .await
        .expect("Failed to connect to test database")
}

// Use hardcoded game IDs (games are now maintained by developers)
const TEST_GAME_ID: &str = "rock-paper-scissors";

#[tokio::test]
async fn test_leaderboard_ordering_and_limit() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("secret".to_string(), 3600);
    // Use hardcoded game (games are now maintained by developers)
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        common::unique_name("Leaderboard Tournament "),
        "Test tournament".to_string(),
        2,
        32,
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
