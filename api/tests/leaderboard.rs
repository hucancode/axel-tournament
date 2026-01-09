mod db;

use api::{
    config::Config, services::{leaderboard, tournament, auth}
};

fn unique_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", prefix, timestamp)
}

// Use hardcoded game IDs (games are now maintained by developers)
const TEST_GAME_ID: &str = "rock-paper-scissors";

async fn get_bob_user(db: &api::db::Database) -> api::models::User {
    let config = Config::from_env();
    auth::get_user_by_email(db, &config.bob.email)
        .await
        .unwrap()
        .expect("Bob user should exist")
}

async fn get_alice_user(db: &api::db::Database) -> api::models::User {
    let config = Config::from_env();
    auth::get_user_by_email(db, &config.alice.email)
        .await
        .unwrap()
        .expect("Alice user should exist")
}

#[tokio::test]
async fn test_leaderboard_ordering_and_limit() {
    let db = db::setup_test_db().await;
    // Create tournament
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

    // Use Bob and Alice users
    let bob_user = get_bob_user(&db).await;
    let alice_user = get_alice_user(&db).await;
    let players = vec![(bob_user, 120.0), (alice_user, 60.0)];

    let mut participants = Vec::new();
    for (player, score) in players {
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
    let db = db::setup_test_db().await;

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

    // Use Bob and Alice users with different scores
    let bob_user = get_bob_user(&db).await;
    let alice_user = get_alice_user(&db).await;
    let players = vec![(bob_user, 100.0), (alice_user, 90.0)];

    let mut participants = Vec::new();
    for (player, score) in players {
        let player_id = player.id.unwrap();
        let participant = tournament::join_tournament(&db, tournament_id.clone(), player_id)
            .await
            .unwrap();
        participants.push((participant.id.unwrap(), score));
    }

    // Assign scores and ranks
    for (i, (participant_id, score)) in participants.iter().enumerate() {
        let participant_id_clone = participant_id.clone();
        let score_clone = *score;
        let rank = i + 1;
        db.query("UPDATE $id SET score = $score, rank = $rank")
            .bind(("id", participant_id_clone))
            .bind(("score", score_clone))
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
}
