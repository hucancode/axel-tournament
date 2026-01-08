use api::{
    config::Config,
    db,
    models::{CreateTournamentRequest, MatchGenerationType, TournamentStatus},
    services::{matches, submission, tournament, auth},
};
use surrealdb::sql::Thing;
use validator::Validate;

async fn setup_test_db() -> api::db::Database {
    let config = Config::from_env();
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
async fn test_create_and_get_tournament() {
    let db = setup_test_db().await;
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        16,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap();
    let fetched = tournament::get_tournament(&db, tournament_id.clone())
        .await
        .unwrap();
    assert_eq!(fetched.name, tournament.name);
    assert_eq!(fetched.status, TournamentStatus::Registration);
}

#[tokio::test]
async fn test_update_tournament_status() {
    let db = setup_test_db().await;
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Status Tournament "),
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
    let updated = tournament::update_tournament(
        &db,
        tournament_id,
        Some("Updated".to_string()),
        None,
        Some(TournamentStatus::Running),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.status, TournamentStatus::Running);
}

#[tokio::test]
async fn test_join_and_leave_tournament() {
    let db = setup_test_db().await;
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Join Tournament "),
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
    
    let bob_user = get_bob_user(&db).await;
    let user_id = bob_user.id.unwrap();
    
    let participant = tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();
    assert_eq!(participant.user_id, user_id);
    
    let participants = tournament::get_tournament_participants(&db, tournament_id.clone())
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    tournament::leave_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();
    let after_leave = tournament::get_tournament_participants(&db, tournament_id)
        .await
        .unwrap();
    assert!(after_leave.is_empty());
}

#[test]
fn test_create_tournament_request_validation() {
    let valid_request = CreateTournamentRequest {
        game_id: TEST_GAME_ID.to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 2,
        max_players: 100,
        start_time: None,
        end_time: None,
        match_generation_type: None,
    };
    assert!(valid_request.validate().is_ok());
    let low_min = CreateTournamentRequest {
        game_id: TEST_GAME_ID.to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 1,
        max_players: 100,
        start_time: None,
        end_time: None,
        match_generation_type: None,
    };
    assert!(low_min.validate().is_err());
    let high_max = CreateTournamentRequest {
        game_id: TEST_GAME_ID.to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 2,
        max_players: 1000,
        start_time: None,
        end_time: None,
        match_generation_type: None,
    };
    assert!(high_max.validate().is_err());
}

#[tokio::test]
async fn test_tournament_status_serialization() {
    let statuses = vec![
        TournamentStatus::Scheduled,
        TournamentStatus::Registration,
        TournamentStatus::Generating,
        TournamentStatus::Running,
        TournamentStatus::Completed,
        TournamentStatus::Cancelled,
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: TournamentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[tokio::test]
async fn test_start_tournament_all_vs_all() {
    let db = setup_test_db().await;

    // Create tournament with AllVsAll match generation (default)
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("AllVsAll Tournament "),
        "Test tournament".to_string(),
        2,
        10,
        None,
        None,
        Some(MatchGenerationType::AllVsAll),
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap();

    // Use Bob and Alice users
    let bob_user = get_bob_user(&db).await;
    let alice_user = get_alice_user(&db).await;
    let user_ids: Vec<Thing> = vec![bob_user.id.unwrap(), alice_user.id.unwrap()];

    // Join tournament and create submissions
    for user_id in &user_ids {
        tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
            .await
            .unwrap();

        submission::create_submission(
            &db,
            user_id.clone(),
            tournament_id.clone(),
            TEST_GAME_ID.to_string(),
            api::models::ProgrammingLanguage::Rust,
            "fn main() {}".to_string(),
        )
        .await
        .unwrap();
    }

    // Start tournament and generate matches
    let started_tournament = tournament::start_tournament(&db, tournament_id.clone())
        .await
        .unwrap();

    // Verify tournament status changed
    assert_eq!(started_tournament.status, TournamentStatus::Running);
    // Verify matches were created (2 players vs 2 players = 4 matches)
    let created_matches =
        matches::list_matches(&db, Some(tournament_id.clone()), None, None, None, None)
            .await
            .unwrap();
    assert_eq!(created_matches.len(), 4);
    assert!(created_matches.iter().all(|m| m.participants.len() == 2));
}

#[tokio::test]
async fn test_start_tournament_round_robin() {
    let db = setup_test_db().await;

    // Create tournament with RoundRobin match generation
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("RoundRobin Tournament "),
        "Test tournament".to_string(),
        2,
        10,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap();

    // Use Bob and Alice users
    let bob_user = get_bob_user(&db).await;
    let alice_user = get_alice_user(&db).await;
    let user_ids: Vec<Thing> = vec![bob_user.id.unwrap(), alice_user.id.unwrap()];

    // Join tournament and create submissions
    for user_id in &user_ids {
        tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
            .await
            .unwrap();

        submission::create_submission(
            &db,
            user_id.clone(),
            tournament_id.clone(),
            TEST_GAME_ID.to_string(),
            api::models::ProgrammingLanguage::Rust,
            "fn main() {}".to_string(),
        )
        .await
        .unwrap();
    }

    // Start tournament and generate matches
    let started_tournament = tournament::start_tournament(&db, tournament_id.clone())
        .await
        .unwrap();

    // Verify tournament status changed
    assert_eq!(started_tournament.status, TournamentStatus::Running);

    // Verify matches were created (2 players, unique pairings = 1 match)
    let created_matches =
        matches::list_matches(&db, Some(tournament_id.clone()), None, None, None, None)
            .await
            .unwrap();
    assert_eq!(created_matches.len(), 1); // 2 * (2-1) / 2 = 1 match (no duplicates)
}

#[tokio::test]
async fn test_start_tournament_without_submissions_fails() {
    let db = setup_test_db().await;

    // Create tournament
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("NoSub Tournament "),
        "Test tournament".to_string(),
        2,
        10,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap();

    // Use Bob and Alice but don't submit code
    let bob_user = get_bob_user(&db).await;
    let alice_user = get_alice_user(&db).await;
    let users = vec![bob_user, alice_user];
    
    for user in users {
        let user_id = user.id.unwrap();
        tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
            .await
            .unwrap();
    }

    // Try to start tournament - should fail because no submissions
    let result = tournament::start_tournament(&db, tournament_id.clone()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_start_tournament_not_enough_players_fails() {
    let db = setup_test_db().await;

    // Create tournament requiring 5 minimum players
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("MinPlayers Tournament "),
        "Test tournament".to_string(),
        5,
        10,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap();

    // Try to start with 0 players - should fail
    let result = tournament::start_tournament(&db, tournament_id.clone()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tournament_participant_management() {
    let db = setup_test_db().await;

    // Create tournament
    let tournament = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Participant Tournament "),
        "Test tournament".to_string(),
        2,
        16,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap();

    // Use Bob user
    let bob_user = get_bob_user(&db).await;
    let user_id = bob_user.id.unwrap();

    // Join tournament
    let participant = tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();
    assert_eq!(participant.user_id, user_id);

    // Verify participant count
    let participants = tournament::get_tournament_participants(&db, tournament_id.clone())
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);

    // Leave tournament
    tournament::leave_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();

    // Verify participant removed
    let after_leave = tournament::get_tournament_participants(&db, tournament_id)
        .await
        .unwrap();
    assert!(after_leave.is_empty());
}
