//! Ordered, idempotent SurrealDB schema migrations. Both api and
//! judge call `run` at startup. Versions are persisted in
//! `schema_migration` so a redeploy never re-runs an applied step,
//! and divergent DDL between services becomes a compile error rather
//! than a runtime drift.

use serde::Deserialize;
use surrealdb::types::SurrealValue;
use surrealdb::{Connection, Surreal};
use tracing::info;

use crate::error::{AppError, AppResult};

pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct AppliedRow {
    version: u32,
}

pub static MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "core_init",
        sql: include_str!("../migrations/001_core_init.surql"),
    },
    Migration {
        version: 2,
        name: "room_tables",
        sql: include_str!("../migrations/002_room_tables.surql"),
    },
    Migration {
        version: 3,
        name: "backfill_legacy",
        sql: include_str!("../migrations/003_backfill_legacy.surql"),
    },
];

/// Re-run every migration's SQL unconditionally. All canonical migrations
/// are idempotent (DEFINE ... IF NOT EXISTS, UPDATE ... WHERE NONE), so
/// this is safe to call from a recurring healer tick. Used to restore
/// schema definitions when an operator drops tables in a live database.
pub async fn ensure_schema<C: Connection>(db: &Surreal<C>) -> AppResult<()> {
    for m in MIGRATIONS {
        db.query(m.sql).await.map_err(|e| {
            AppError::Internal(format!(
                "ensure_schema {} ({}): {}",
                m.version, m.name, e
            ))
        })?;
    }
    Ok(())
}

pub async fn run<C: Connection>(db: &Surreal<C>) -> AppResult<()> {
    db.query(
        "DEFINE TABLE IF NOT EXISTS schema_migration SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS version ON schema_migration TYPE number;
         DEFINE FIELD IF NOT EXISTS name ON schema_migration TYPE string;
         DEFINE FIELD IF NOT EXISTS applied_at ON schema_migration TYPE datetime;
         DEFINE INDEX IF NOT EXISTS unique_version ON schema_migration COLUMNS version UNIQUE;",
    )
    .await?;

    let rows: Vec<AppliedRow> = db
        .query("SELECT version FROM schema_migration")
        .await?
        .take(0)?;
    let applied: std::collections::HashSet<u32> = rows.into_iter().map(|r| r.version).collect();

    for m in MIGRATIONS {
        if applied.contains(&m.version) {
            continue;
        }
        info!(version = m.version, name = m.name, "applying migration");
        db.query(m.sql).await.map_err(|e| {
            AppError::Internal(format!(
                "migration {} ({}) failed: {}",
                m.version, m.name, e
            ))
        })?;
        db.query(
            "CREATE schema_migration CONTENT { version: $v, name: $n, applied_at: time::now() }",
        )
        .bind(("v", m.version as i64))
        .bind(("n", m.name.to_string()))
        .await?;
    }
    Ok(())
}
