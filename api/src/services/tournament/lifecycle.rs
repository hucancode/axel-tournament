use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::tournament::{
        MatchGenerationType, Tournament, TournamentKind, TournamentParticipant, TournamentStatus,
    },
    services::common::enum_tag,
};
use chrono::{DateTime, Utc};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

use super::generation;

#[allow(clippy::too_many_arguments)]
pub async fn create_tournament(
    db: &Database,
    game_id: String,
    name: String,
    description: String,
    min_players: u32,
    max_players: u32,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    match_generation_type: Option<MatchGenerationType>,
    kind: Option<TournamentKind>,
    config: Option<serde_json::Value>,
) -> ApiResult<Tournament> {
    let config = config.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let game = crate::models::game::find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    // Score-based games (rps, pd, poker) accumulate score; bracket
    // elimination is incompatible with that — losers would drop out
    // before their score had time to converge. Default them to AllVsAll
    // and reject explicit elim choices.
    let mgt = match (game.scoring_kind, match_generation_type) {
        (crate::models::game::ScoringKind::Score,
         Some(MatchGenerationType::SingleElimination | MatchGenerationType::DoubleElimination)) => {
            return Err(ApiError::Validation(
                "Score-based games cannot use elimination brackets".to_string(),
            ));
        }
        (crate::models::game::ScoringKind::Score, None) => MatchGenerationType::AllVsAll,
        (_, Some(m)) => m,
        (_, None) => MatchGenerationType::default(),
    };

    let tournament = Tournament {
        id: None,
        game_id,
        name,
        description,
        status: TournamentStatus::Registration,
        min_players,
        max_players,
        start_time: start_time.map(|dt| dt.into()),
        end_time: end_time.map(|dt| dt.into()),
        match_generation_type: mgt,
        kind: kind.unwrap_or_default(),
        config,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Tournament> = db.create("tournament").content(tournament).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create tournament".to_string()))
}

pub async fn get_tournament(db: &Database, tournament_id: RecordId) -> ApiResult<Tournament> {
    let tournament: Option<Tournament> = db.select(&tournament_id).await?;
    tournament.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

pub async fn list_tournaments(
    db: &Database,
    status: Option<TournamentStatus>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> ApiResult<Vec<(Tournament, u32)>> {
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);
    let mut result = if let Some(s) = status {
        db.query(
            "LET $ts = (SELECT * FROM tournament
                         WHERE status = $status
                         ORDER BY created_at DESC
                         LIMIT $limit START $offset);
             RETURN $ts;
             SELECT tournament_id, count() AS n FROM tournament_participant
             WHERE tournament_id IN $ts.id
             GROUP BY tournament_id",
        )
        .bind(("status", enum_tag(&s)))
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await?
    } else {
        db.query(
            "LET $ts = (SELECT * FROM tournament
                         ORDER BY created_at DESC
                         LIMIT $limit START $offset);
             RETURN $ts;
             SELECT tournament_id, count() AS n FROM tournament_participant
             WHERE tournament_id IN $ts.id
             GROUP BY tournament_id",
        )
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await?
    };
    let tournaments: Vec<Tournament> = result.take(1)?;
    #[derive(serde::Deserialize, SurrealValue)]
    struct CountRow {
        tournament_id: RecordId,
        n: i64,
    }
    let count_rows: Vec<CountRow> = result.take(2).unwrap_or_default();
    let counts: std::collections::HashMap<RecordId, u32> = count_rows
        .into_iter()
        .map(|r| (r.tournament_id, r.n.max(0) as u32))
        .collect();
    Ok(tournaments
        .into_iter()
        .map(|t| {
            let c = t
                .id
                .as_ref()
                .and_then(|id| counts.get(id).copied())
                .unwrap_or(0);
            (t, c)
        })
        .collect())
}

pub async fn update_tournament(
    db: &Database,
    tournament_id: RecordId,
    name: Option<String>,
    description: Option<String>,
    status: Option<TournamentStatus>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> ApiResult<Tournament> {
    let mut tournament = get_tournament(db, tournament_id.clone()).await?;
    if let Some(n) = name {
        tournament.name = n;
    }
    if let Some(d) = description {
        tournament.description = d;
    }
    if let Some(s) = status {
        tournament.status = s;
    }
    if let Some(st) = start_time {
        tournament.start_time = Some(st.into());
    }
    if let Some(et) = end_time {
        tournament.end_time = Some(et.into());
    }
    tournament.updated_at = Datetime::default();
    let updated: Option<Tournament> = db.update(&tournament_id).content(tournament).await?;
    updated.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

/// Replace a tournament's per-game `config` blob. Only allowed before
/// the tournament transitions out of registration.
pub async fn update_tournament_config(
    db: &Database,
    tournament_id: RecordId,
    config: serde_json::Value,
) -> ApiResult<Tournament> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration
        && tournament.status != TournamentStatus::Scheduled
    {
        return Err(ApiError::BadRequest(
            "Cannot edit config after tournament has started".to_string(),
        ));
    }
    let mut resp = db
        .query(
            "UPDATE $tid SET config = $cfg, updated_at = time::now()
             RETURN AFTER",
        )
        .bind(("tid", tournament_id.clone()))
        .bind(("cfg", config))
        .await?;
    let rows: Vec<Tournament> = resp.take(0)?;
    rows.into_iter()
        .next()
        .ok_or_else(|| ApiError::Internal("Failed to update tournament config".to_string()))
}

pub async fn join_tournament(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> ApiResult<TournamentParticipant> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Tournament is not accepting registrations".to_string(),
        ));
    }

    let participants = get_tournament_participants(db, tournament_id.clone()).await?;
    if participants.len() as u32 >= tournament.max_players {
        return Err(ApiError::BadRequest("Tournament is full".to_string()));
    }
    if participants.iter().any(|p| p.user_id == user_id) {
        return Err(ApiError::Conflict(
            "User already joined this tournament".to_string(),
        ));
    }

    let participant = TournamentParticipant {
        id: None,
        tournament_id: tournament_id.clone(),
        user_id: user_id.clone(),
        submission_id: None,
        score: 0.0,
        wins: 0,
        losses: 0,
        draws: 0,
        elo: None,
        rank: None,
        joined_at: Datetime::default(),
    };
    let created: Option<TournamentParticipant> = db
        .create("tournament_participant")
        .content(participant)
        .await?;
    created.ok_or_else(|| ApiError::Internal("Failed to join tournament".to_string()))
}

pub async fn get_tournament_participants(
    db: &Database,
    tournament_id: RecordId,
) -> ApiResult<Vec<TournamentParticipant>> {
    let mut result = db
        .query(
            "SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id ORDER BY score DESC",
        )
        .bind(("tournament_id", tournament_id))
        .await?;
    let participants: Vec<TournamentParticipant> = result.take(0)?;
    Ok(participants)
}

pub async fn get_tournament_participants_with_usernames(
    db: &Database,
    tournament_id: RecordId,
) -> ApiResult<Vec<(TournamentParticipant, Option<String>)>> {
    let participants = get_tournament_participants(db, tournament_id).await?;
    if participants.is_empty() {
        return Ok(Vec::new());
    }
    let user_ids: Vec<RecordId> = participants.iter().map(|p| p.user_id.clone()).collect();
    let mut response = db
        .query("SELECT id, username FROM user WHERE id IN $ids")
        .bind(("ids", user_ids))
        .await?;

    #[derive(serde::Deserialize, SurrealValue)]
    struct UserRow {
        id: RecordId,
        username: Option<String>,
    }
    let rows: Vec<UserRow> = response.take(0)?;
    let mut by_id: std::collections::HashMap<RecordId, String> = rows
        .into_iter()
        .filter_map(|r| r.username.map(|u| (r.id, u)))
        .collect();
    Ok(participants
        .into_iter()
        .map(|p| {
            let username = by_id.remove(&p.user_id);
            (p, username)
        })
        .collect())
}

/// Single participant lookup by `(tournament, user)`. None if absent.
pub async fn get_participant(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
) -> ApiResult<Option<TournamentParticipant>> {
    let mut resp = db
        .query(
            "SELECT * FROM tournament_participant
             WHERE tournament_id = $tid AND user_id = $uid LIMIT 1",
        )
        .bind(("tid", tournament_id.clone()))
        .bind(("uid", user_id.clone()))
        .await?;
    let rows: Vec<TournamentParticipant> = resp.take(0)?;
    Ok(rows.into_iter().next())
}

pub async fn leave_tournament(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> ApiResult<()> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Cannot leave tournament after registration has closed".to_string(),
        ));
    }
    let participant = get_participant(db, &tournament_id, &user_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound("You are not a participant in this tournament".to_string())
        })?;

    if let Some(pid) = participant.id.clone() {
        let _: Option<TournamentParticipant> = db.delete(&pid).await?;
    }
    Ok(())
}

/// Roll back a failed tournament start: delete generated matches and put
/// the tournament back into `registration`.
async fn rollback_tournament_start(db: &Database, tournament_id: &RecordId) {
    let _ = db
        .query("DELETE match WHERE tournament_id = $tournament_id")
        .bind(("tournament_id", tournament_id.clone()))
        .await;
    let _ = db
        .query(
            "UPDATE $tournament_id
             SET status = 'registration', updated_at = time::now()",
        )
        .bind(("tournament_id", tournament_id.clone()))
        .await;
}

/// Start a tournament and generate matches based on the configured match
/// generation type.
pub async fn start_tournament(
    db: &Database,
    tournament_id: RecordId,
) -> ApiResult<Tournament> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Tournament must be in registration state to start".to_string(),
        ));
    }

    let participants = get_tournament_participants(db, tournament_id.clone()).await?;
    if tournament.min_players > participants.len() as u32 {
        return Err(ApiError::BadRequest(format!(
            "Not enough players. Need at least {} players, currently have {}",
            tournament.min_players,
            participants.len()
        )));
    }

    // Filter to participants whose selected submission compiled cleanly.
    // One DB roundtrip via id-list, not N selects.
    let participants_with_submissions =
        compiled_participants(db, &participants).await?;

    if participants_with_submissions.is_empty() {
        return Err(ApiError::BadRequest(
            "No participants with a compiled submission. Wait for compilation to finish or upload a fixed bot.".to_string(),
        ));
    }
    if (participants_with_submissions.len() as u32) < tournament.min_players {
        return Err(ApiError::BadRequest(format!(
            "Not enough compiled submissions. Need {}, have {}.",
            tournament.min_players,
            participants_with_submissions.len()
        )));
    }

    // Claim tournament start to prevent duplicate match generation.
    let mut claimed = db
        .query(
            "UPDATE $tournament_id
             SET status = 'generating', updated_at = time::now()
             WHERE status = 'registration'
             RETURN AFTER",
        )
        .bind(("tournament_id", tournament_id.clone()))
        .await?;
    let claimed_rows: Vec<Tournament> = claimed.take(0)?;
    if claimed_rows.is_empty() {
        return Err(ApiError::BadRequest(
            "Tournament has already been started".to_string(),
        ));
    }

    let matches_created = match generation::generate_matches(
        db,
        &tournament,
        &participants_with_submissions,
    )
    .await
    {
        Ok(count) => count,
        Err(err) => {
            rollback_tournament_start(db, &tournament_id).await;
            return Err(err);
        }
    };

    if matches_created == 0 {
        rollback_tournament_start(db, &tournament_id).await;
        return Err(ApiError::Internal(
            "No matches were generated for this tournament".to_string(),
        ));
    }

    let mut updated = db
        .query(
            "UPDATE $tournament_id
             SET status = 'running', updated_at = time::now()
             WHERE status = 'generating'
             RETURN AFTER",
        )
        .bind(("tournament_id", tournament_id.clone()))
        .await?;
    let updated_rows: Vec<Tournament> = updated.take(0)?;
    if let Some(t) = updated_rows.into_iter().next() {
        return Ok(t);
    }
    get_tournament(db, tournament_id).await
}

/// Filter `participants` to those whose selected submission compiled
/// (`status = 'accepted'`). One DB roundtrip with `WHERE id IN $ids`.
async fn compiled_participants(
    db: &Database,
    participants: &[TournamentParticipant],
) -> ApiResult<Vec<TournamentParticipant>> {
    let sub_ids: Vec<RecordId> = participants
        .iter()
        .filter_map(|p| p.submission_id.clone())
        .collect();
    if sub_ids.is_empty() {
        return Ok(Vec::new());
    }
    use surrealdb::types::SurrealValue;
    #[derive(serde::Deserialize, SurrealValue)]
    struct AcceptedRow {
        id: RecordId,
    }
    let mut resp = db
        .query("SELECT id FROM submission WHERE id IN $ids AND status = 'accepted'")
        .bind(("ids", sub_ids))
        .await?;
    let rows: Vec<AcceptedRow> = resp.take(0)?;
    let accepted: Vec<RecordId> = rows.into_iter().map(|r| r.id).collect();
    Ok(participants
        .iter()
        .filter(|p| {
            p.submission_id
                .as_ref()
                .is_some_and(|sid| accepted.contains(sid))
        })
        .cloned()
        .collect())
}
