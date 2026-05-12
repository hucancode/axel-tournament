use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::tournament::{
        MatchGenerationType, Tournament, TournamentKind, TournamentStatus,
    },
    services::common::enum_tag,
};
use axel_core::repo::tournament::TournamentRepo;
use chrono::{DateTime, Utc};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

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
) -> AppResult<Tournament> {
    if min_players > max_players {
        return Err(AppError::Validation(
            "min_players cannot be greater than max_players".to_string(),
        ));
    }
    let config = config.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let game = crate::models::game::find_game_by_id(&game_id)
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))?;

    // Score-based games (rps, pd, poker) accumulate score; bracket
    // elimination is incompatible with that — losers would drop out
    // before their score had time to converge. Default them to AllVsAll
    // and reject explicit elim choices.
    let mgt = match (game.scoring_kind, match_generation_type) {
        (crate::models::game::ScoringKind::Score,
         Some(MatchGenerationType::SingleElimination | MatchGenerationType::DoubleElimination)) => {
            return Err(AppError::Validation(
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
    <Database as TournamentRepo>::create(db, tournament).await
}

pub async fn get_tournament(db: &Database, tournament_id: RecordId) -> AppResult<Tournament> {
    <Database as TournamentRepo>::get_by_id(db, &tournament_id).await
}

pub async fn list_tournaments(
    db: &Database,
    status: Option<TournamentStatus>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> AppResult<Vec<(Tournament, u32)>> {
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
) -> AppResult<Tournament> {
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
    <Database as TournamentRepo>::replace(db, &tournament_id, tournament).await
}

/// Replace a tournament's per-game `config` blob. Only allowed before
/// the tournament transitions out of registration.
pub async fn update_tournament_config(
    db: &Database,
    tournament_id: RecordId,
    config: serde_json::Value,
) -> AppResult<Tournament> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration
        && tournament.status != TournamentStatus::Scheduled
    {
        return Err(AppError::BadRequest(
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
        .ok_or_else(|| AppError::Internal("Failed to update tournament config".to_string()))
}
