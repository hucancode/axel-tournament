use anyhow::Result;
use judge::config::Config;
use judge::games::Game;
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct SubmissionStatus {
    status: String,
    compiled_binary_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Participant {
    score: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct MatchStatus {
    status: String,
    participants: Option<Vec<Participant>>,
    error_message: Option<String>,
}

async fn connect_db() -> Result<Surreal<Client>> {
    dotenv::dotenv().ok();
    let config = Config::from_env()?;

    use judge::db::connect;
    connect(
        &config.database_url,
        &config.database_ns,
        &config.database_db,
        &config.database_user,
        &config.database_pass,
    ).await
}

async fn create_test_submission(db: &Surreal<Client>, tournament_id: &str, code: String) -> Result<Thing> {
    // Add a small delay to avoid race conditions when creating multiple submissions
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

    // First create a test tournament (use UPSERT to handle existing records)
    let _tournament_result = db.query(
        "UPSERT $tournament_id SET
            name = 'Test Tournament',
            description = 'Test tournament for E2E testing',
            game_id = 'rock-paper-scissors',
            status = 'registration',
            min_players = 2,
            max_players = 10,
            match_generation_type = 'AllVsAll',
            created_at = time::now(),
            updated_at = time::now()"
    )
    .bind(("tournament_id", Thing::from(("tournament", tournament_id))))
    .await?;

    // Generate a unique submission ID
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let submission_id = format!("test_submission_{}", timestamp);
    let submission_thing = Thing::from(("submission", submission_id.as_str()));

    db.query(
        "CREATE $submission_id SET
            user_id = user:alice,
            tournament_id = $tournament_id,
            game_id = 'rock-paper-scissors',
            language = 'rust',
            code = $code,
            status = 'pending',
            created_at = time::now(),
            updated_at = time::now()"
    )
    .bind(("submission_id", submission_thing.clone()))
    .bind(("tournament_id", Thing::from(("tournament", tournament_id))))
    .bind(("code", code))
    .await?;
    Ok(submission_thing)
}

async fn create_test_match(db: &Surreal<Client>, submission1_id: Thing, submission2_id: Thing, tournament_id: &str) -> Result<Thing> {
    // Generate a unique match ID
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let match_id = format!("test_match_{}", timestamp);
    let match_thing = Thing::from(("match", match_id.as_str()));

    db.query(
        "CREATE $match_id SET
            tournament_id = $tournament_id,
            game_id = 'rock-paper-scissors',
            status = 'pending',
            participants = [
                { submission_id: $sub1 },
                { submission_id: $sub2 }
            ],
            created_at = time::now(),
            updated_at = time::now()"
    )
    .bind(("match_id", match_thing.clone()))
    .bind(("tournament_id", Thing::from(("tournament", tournament_id))))
    .bind(("sub1", submission1_id))
    .bind(("sub2", submission2_id))
    .await?;

    Ok(match_thing)
}

async fn wait_for_match_completed(db: &Surreal<Client>, match_id: Thing) -> Result<MatchStatus> {
    for _ in 0..20 {
        let mut result = db.query("SELECT status, participants, error_message FROM $match_id")
            .bind(("match_id", match_id.clone()))
            .await?;
        let matches: Vec<MatchStatus> = result.take(0)?;

        if let Some(m) = matches.first() {
            if m.status == "completed" {
                return Ok(m.clone());
            } else if m.status == "failed" {
                let error_msg = m.error_message.as_deref().unwrap_or("Unknown error");
                anyhow::bail!("Match execution failed: {}", error_msg);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    anyhow::bail!("Timeout waiting for match completion")
}

#[tokio::test]
async fn test_e2e_judge_workflow() -> Result<()> {
    // Connect to database
    let db = connect_db().await?;

    // Generate unique tournament ID for this test
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos(); // Use nanoseconds for better uniqueness
    let tournament_id = format!("test_tournament_e2e_{}", timestamp);

    // Step 1: Create mock code submissions
    println!("Step 1: Creating test submissions...");
    let rock_bot_code = include_str!("bots/rps_rock.rs");
    let paper_bot_code = include_str!("bots/rps_paper.rs");
    let submission1_id = create_test_submission(&db, &tournament_id, rock_bot_code.to_string()).await?;
    let submission2_id = create_test_submission(&db, &tournament_id, paper_bot_code.to_string()).await?;
    println!("Created submissions: {} and {}", submission1_id, submission2_id);

    // Step 2: Create match request (submissions will be compiled on-demand by judge)
    println!("Step 2: Creating match request...");
    let match_id = create_test_match(&db, submission1_id.clone(), submission2_id.clone(), &tournament_id).await?;
    println!("Created match: {}", match_id);

    // Step 3: Start judge server and wait for match processing
    println!("Step 3: Starting judge server...");

    // Import and use judge's match watcher directly
    use judge::games::RockPaperScissors;
    use judge::match_watcher::start_match_watcher;
    use judge::capacity::CapacityTracker;

    let capacity = CapacityTracker::new(10, 100);
    let game = RockPaperScissors::new();

    // Start match watcher in background using existing db connection
    let judge_db_clone = db.clone();
    let capacity_clone = capacity.clone();
    let watcher_handle = tokio::spawn(async move {
        tokio::select! {
            result = start_match_watcher(
                judge_db_clone,
                game,
                "rock-paper-scissors".to_string(),
                capacity_clone,
            ) => {
                if let Err(e) = result {
                    panic!("Match watcher error: {}", e);
                }
            }
            _ = sleep(Duration::from_secs(30)) => {
                panic!("Match watcher timeout after 30 seconds");
            }
        }
    });

    // Give the watcher a moment to start
    sleep(Duration::from_millis(100)).await;

    // Step 4: Wait for judge to process match (including compilation)
    println!("Step 4: Waiting for judge to process match...");
    let completed_match = wait_for_match_completed(&db, match_id.clone()).await?;

    // Step 5: Verify results
    println!("Step 5: Verifying match results...");
    assert_eq!(completed_match.status, "completed");

    let participants = completed_match.participants.as_ref().unwrap();
    assert_eq!(participants.len(), 2);
    assert!(participants.iter().all(|it| it.score.is_some()));
    // Verify submissions were compiled during match execution
    let mut result1 = db.query("SELECT status, compiled_binary_path FROM $submission_id")
        .bind(("submission_id", submission1_id.clone()))
        .await?;
    let submissions1: Vec<SubmissionStatus> = result1.take(0)?;
    let sub1 = submissions1.first().expect("Submission 1 not found");
    assert_eq!(sub1.status, "accepted");
    assert!(sub1.compiled_binary_path.is_some());

    let mut result2 = db.query("SELECT status, compiled_binary_path FROM $submission_id")
        .bind(("submission_id", submission2_id.clone()))
        .await?;
    let submissions2: Vec<SubmissionStatus> = result2.take(0)?;
    let sub2 = submissions2.first().expect("Submission 2 not found");
    assert_eq!(sub2.status, "accepted");
    assert!(sub2.compiled_binary_path.is_some());
    println!("Match completed successfully!");
    println!("Scores: {:?}",
        participants.iter().map(|p| p.score).collect::<Vec<_>>());
    watcher_handle.abort();
    db.query("DELETE $submission1_id")
        .bind(("submission1_id", submission1_id))
        .await?;
    db.query("DELETE $submission2_id")
        .bind(("submission2_id", submission2_id))
        .await?;
    db.query("DELETE $match_id")
        .bind(("match_id", match_id))
        .await?;
    db.query("DELETE $tournament_id")
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_e2e_c_compilation() -> Result<()> {
    let db = connect_db().await?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tournament_id = format!("test_tournament_c_{}", timestamp);

    println!("Step 1: Creating C test submissions...");
    let rock_bot_code = include_str!("bots/rps_rock.c");
    let paper_bot_code = include_str!("bots/rps_paper.c");

    let submission1_thing = {
        let submission_id = format!("test_c_submission_{}_{}", timestamp, 1);
        let thing = Thing::from(("submission", submission_id.as_str()));
        db.query(
            "CREATE $submission_id SET
                user_id = user:alice,
                tournament_id = $tournament_id,
                game_id = 'rock-paper-scissors',
                language = 'c',
                code = $code,
                status = 'pending',
                created_at = time::now(),
                updated_at = time::now()"
        )
        .bind(("submission_id", thing.clone()))
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .bind(("code", rock_bot_code.to_string()))
        .await?;
        thing
    };

    let submission2_thing = {
        let submission_id = format!("test_c_submission_{}_{}", timestamp, 2);
        let thing = Thing::from(("submission", submission_id.as_str()));
        db.query(
            "CREATE $submission_id SET
                user_id = user:bob,
                tournament_id = $tournament_id,
                game_id = 'rock-paper-scissors',
                language = 'c',
                code = $code,
                status = 'pending',
                created_at = time::now(),
                updated_at = time::now()"
        )
        .bind(("submission_id", thing.clone()))
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .bind(("code", paper_bot_code.to_string()))
        .await?;
        thing
    };

    println!("Created C submissions: {} and {}", submission1_thing, submission2_thing);

    println!("Step 2: Creating match request...");
    let match_id = create_test_match(&db, submission1_thing.clone(), submission2_thing.clone(), &tournament_id).await?;
    println!("Created match: {}", match_id);

    println!("Step 3: Starting judge server...");
    use judge::games::RockPaperScissors;
    use judge::match_watcher::start_match_watcher;
    use judge::capacity::CapacityTracker;

    let capacity = CapacityTracker::new(10, 100);
    let game = RockPaperScissors::new();
    let judge_db_clone = db.clone();
    let capacity_clone = capacity.clone();

    let watcher_handle = tokio::spawn(async move {
        tokio::select! {
            result = start_match_watcher(
                judge_db_clone,
                game,
                "rock-paper-scissors".to_string(),
                capacity_clone,
            ) => {
                if let Err(e) = result {
                    panic!("Match watcher error: {}", e);
                }
            }
            _ = sleep(Duration::from_secs(30)) => {
                panic!("Match watcher timeout after 30 seconds");
            }
        }
    });

    sleep(Duration::from_millis(100)).await;

    println!("Step 4: Waiting for judge to process match...");
    let completed_match = wait_for_match_completed(&db, match_id.clone()).await?;

    println!("Step 5: Verifying C compilation and match results...");
    assert_eq!(completed_match.status, "completed");

    let participants = completed_match.participants.as_ref().unwrap();
    assert_eq!(participants.len(), 2);
    assert!(participants.iter().all(|it| it.score.is_some()));

    let mut result1 = db.query("SELECT status, compiled_binary_path FROM $submission_id")
        .bind(("submission_id", submission1_thing.clone()))
        .await?;
    let submissions1: Vec<SubmissionStatus> = result1.take(0)?;
    let sub1 = submissions1.first().expect("C submission 1 not found");
    assert_eq!(sub1.status, "accepted");
    assert!(sub1.compiled_binary_path.is_some());

    let mut result2 = db.query("SELECT status, compiled_binary_path FROM $submission_id")
        .bind(("submission_id", submission2_thing.clone()))
        .await?;
    let submissions2: Vec<SubmissionStatus> = result2.take(0)?;
    let sub2 = submissions2.first().expect("C submission 2 not found");
    assert_eq!(sub2.status, "accepted");
    assert!(sub2.compiled_binary_path.is_some());

    println!("C compilation and match completed successfully!");
    watcher_handle.abort();

    db.query("DELETE $submission1_id")
        .bind(("submission1_id", submission1_thing))
        .await?;
    db.query("DELETE $submission2_id")
        .bind(("submission2_id", submission2_thing))
        .await?;
    db.query("DELETE $match_id")
        .bind(("match_id", match_id))
        .await?;
    db.query("DELETE $tournament_id")
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .await?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_e2e_go_compilation() -> Result<()> {
    let db = connect_db().await?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tournament_id = format!("test_tournament_go_{}", timestamp);

    println!("Step 1: Creating Go test submissions...");
    let rock_bot_code = include_str!("bots/rps_rock.go");
    let paper_bot_code = include_str!("bots/rps_paper.go");

    let submission1_thing = {
        let submission_id = format!("test_go_submission_{}_{}", timestamp, 1);
        let thing = Thing::from(("submission", submission_id.as_str()));
        db.query(
            "CREATE $submission_id SET
                user_id = user:alice,
                tournament_id = $tournament_id,
                game_id = 'rock-paper-scissors',
                language = 'go',
                code = $code,
                status = 'pending',
                created_at = time::now(),
                updated_at = time::now()"
        )
        .bind(("submission_id", thing.clone()))
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .bind(("code", rock_bot_code.to_string()))
        .await?;
        thing
    };

    let submission2_thing = {
        let submission_id = format!("test_go_submission_{}_{}", timestamp, 2);
        let thing = Thing::from(("submission", submission_id.as_str()));
        db.query(
            "CREATE $submission_id SET
                user_id = user:bob,
                tournament_id = $tournament_id,
                game_id = 'rock-paper-scissors',
                language = 'go',
                code = $code,
                status = 'pending',
                created_at = time::now(),
                updated_at = time::now()"
        )
        .bind(("submission_id", thing.clone()))
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .bind(("code", paper_bot_code.to_string()))
        .await?;
        thing
    };

    println!("Created Go submissions: {} and {}", submission1_thing, submission2_thing);

    println!("Step 2: Creating match request...");
    let match_id = create_test_match(&db, submission1_thing.clone(), submission2_thing.clone(), &tournament_id).await?;
    println!("Created match: {}", match_id);

    println!("Step 3: Starting judge server...");
    use judge::games::RockPaperScissors;
    use judge::match_watcher::start_match_watcher;
    use judge::capacity::CapacityTracker;

    let capacity = CapacityTracker::new(10, 100);
    let game = RockPaperScissors::new();
    let judge_db_clone = db.clone();
    let capacity_clone = capacity.clone();

    let watcher_handle = tokio::spawn(async move {
        tokio::select! {
            result = start_match_watcher(
                judge_db_clone,
                game,
                "rock-paper-scissors".to_string(),
                capacity_clone,
            ) => {
                if let Err(e) = result {
                    panic!("Match watcher error: {}", e);
                }
            }
            _ = sleep(Duration::from_secs(30)) => {
                panic!("Match watcher timeout after 30 seconds");
            }
        }
    });

    sleep(Duration::from_millis(100)).await;

    println!("Step 4: Waiting for judge to process match...");
    let completed_match = wait_for_match_completed(&db, match_id.clone()).await?;

    println!("Step 5: Verifying Go compilation and match results...");
    assert_eq!(completed_match.status, "completed");

    let participants = completed_match.participants.as_ref().unwrap();
    assert_eq!(participants.len(), 2);
    assert!(participants.iter().all(|it| it.score.is_some()));

    let mut result1 = db.query("SELECT status, compiled_binary_path FROM $submission_id")
        .bind(("submission_id", submission1_thing.clone()))
        .await?;
    let submissions1: Vec<SubmissionStatus> = result1.take(0)?;
    let sub1 = submissions1.first().expect("Go submission 1 not found");
    assert_eq!(sub1.status, "accepted");
    assert!(sub1.compiled_binary_path.is_some());

    let mut result2 = db.query("SELECT status, compiled_binary_path FROM $submission_id")
        .bind(("submission_id", submission2_thing.clone()))
        .await?;
    let submissions2: Vec<SubmissionStatus> = result2.take(0)?;
    let sub2 = submissions2.first().expect("Go submission 2 not found");
    assert_eq!(sub2.status, "accepted");
    assert!(sub2.compiled_binary_path.is_some());

    println!("Go compilation and match completed successfully!");
    watcher_handle.abort();

    db.query("DELETE $submission1_id")
        .bind(("submission1_id", submission1_thing))
        .await?;
    db.query("DELETE $submission2_id")
        .bind(("submission2_id", submission2_thing))
        .await?;
    db.query("DELETE $match_id")
        .bind(("match_id", match_id))
        .await?;
    db.query("DELETE $tournament_id")
        .bind(("tournament_id", Thing::from(("tournament", tournament_id.as_str()))))
        .await?;

    Ok(())
}
