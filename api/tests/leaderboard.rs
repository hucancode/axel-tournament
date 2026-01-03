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

fn unique_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", prefix, timestamp)
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
        unique_name("Leaderboard Tournament "),
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
            format!("{}@test.com", unique_name("leaderboard_user")),
            unique_name("player"),
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

#[tokio::test]
async fn test_tournament_specific_leaderboard() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("secret".to_string(), 3600);
    
    // Create tournament
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Specific Leaderboard Tournament "),
        "Test tournament".to_string(),
        2,
        8,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap();
    
    // Create multiple players with different scores
    let mut participants = Vec::new();
    for (i, score) in [100.0, 90.0].iter().enumerate() {
        let password_hash = auth_service.hash_password("password123").unwrap();
        let player = user::create_user(
            &db,
            format!("{}@test.com", unique_name(&format!("lb_player_{}", i))),
            unique_name(&format!("lb_player_{}", i)),
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
        participants.push((participant.id.unwrap(), *score, i + 1));
    }
    
    // Assign scores and ranks
    for (participant_id, score, rank) in participants {
        db.query("UPDATE $id SET score = $score, rank = $rank")
            .bind(("id", participant_id))
            .bind(("score", score))
            .bind(("rank", rank))
            .await
            .unwrap();
    }
    
    // Get leaderboard for specific tournament
    let leaderboard_entries = leaderboard::get_leaderboard(&db, 10, Some(tournament_id.clone()), None)
        .await
        .unwrap();
    
    assert_eq!(leaderboard_entries.len(), 2);
    // Verify ordering (highest score first)
    assert!(leaderboard_entries[0].score >= leaderboard_entries[1].score);
    assert_eq!(leaderboard_entries[0].rank, 1);
    assert_eq!(leaderboard_entries[1].rank, 2);
}
