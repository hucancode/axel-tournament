use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::MatchPolicy,
};
use surrealdb::sql::Thing;

pub async fn create_policy(
    db: &Database,
    tournament_id: String,
    rounds_per_match: u32,
    repetitions: u32,
    timeout_seconds: u32,
    cpu_limit: Option<String>,
    memory_limit: Option<String>,
    scoring_weights: Option<serde_json::Value>,
) -> ApiResult<MatchPolicy> {
    let tournament_id_thing = Thing::from(("tournament", tournament_id.trim_start_matches("tournament:")));

    let policy = MatchPolicy {
        id: None,
        tournament_id: tournament_id_thing,
        rounds_per_match,
        repetitions,
        timeout_seconds,
        cpu_limit,
        memory_limit,
        scoring_weights,
    };

    let created: Option<MatchPolicy> = db.create("match_policy").content(policy).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create match policy".to_string()))
}

pub async fn get_policy(db: &Database, tournament_id: &str) -> ApiResult<MatchPolicy> {
    let tournament_id_clean = tournament_id.trim_start_matches("tournament:");
    let mut result = db
        .query("SELECT * FROM match_policy WHERE tournament_id = $tournament_id")
        .bind(("tournament_id", Thing::from(("tournament", tournament_id_clean))))
        .await?;

    let policies: Vec<MatchPolicy> = result.take(0)?;
    policies.into_iter().next()
        .ok_or_else(|| ApiError::NotFound("Match policy not found".to_string()))
}

pub async fn update_policy(
    db: &Database,
    tournament_id: &str,
    rounds_per_match: Option<u32>,
    repetitions: Option<u32>,
    timeout_seconds: Option<u32>,
    cpu_limit: Option<String>,
    memory_limit: Option<String>,
    scoring_weights: Option<serde_json::Value>,
) -> ApiResult<MatchPolicy> {
    let mut policy = get_policy(db, tournament_id).await?;

    if let Some(r) = rounds_per_match { policy.rounds_per_match = r; }
    if let Some(rep) = repetitions { policy.repetitions = rep; }
    if let Some(t) = timeout_seconds { policy.timeout_seconds = t; }
    if cpu_limit.is_some() { policy.cpu_limit = cpu_limit; }
    if memory_limit.is_some() { policy.memory_limit = memory_limit; }
    if scoring_weights.is_some() { policy.scoring_weights = scoring_weights; }

    let policy_id = policy.id.clone().unwrap();
    let updated: Option<MatchPolicy> = db
        .update((policy_id.tb.as_str(), policy_id.id.to_string()))
        .content(policy)
        .await?;

    updated.ok_or_else(|| ApiError::Internal("Failed to update policy".to_string()))
}
