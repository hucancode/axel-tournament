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
        url: "ws://127.0.0.1:8001".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: namespace.clone(),
        database: namespace,
    };
    db::connect(&config)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_leaderboard_ordering_and_limit() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("secret".to_string(), 3600);
    let game = game::create_game(
        &db,
        common::unique_name("Leaderboard Game "),
        "Desc".to_string(),
        serde_json::json!({}),
        vec![ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();
    let tournament = tournament::create_tournament(
        &db,
        game_id,
        common::unique_name("Leaderboard Tournament "),
        "Test tournament".to_string(),
        2,
        32,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap().id.to_string();
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
        let player_id = player.id.unwrap().id.to_string();
        let participant = tournament::join_tournament(&db, &tournament_id, &player_id)
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
    let leaderboard_entries = leaderboard::get_leaderboard(&db, 1, Some(&tournament_id), None)
        .await
        .unwrap();
    assert_eq!(leaderboard_entries.len(), 1);
    assert!(leaderboard_entries[0].score >= 100.0);
    assert_eq!(leaderboard_entries[0].rank, 1);
}
