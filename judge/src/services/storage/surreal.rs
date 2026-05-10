// SurrealDB-backed storage. Production.
//
// Schema (see judge/protocols/architecture.md):
//   room_event: { id, room (string), seq (int), kind, payload, ts }
//   room_lease: { id (= room_id), owner (string), expires (datetime) }
//   room_meta:  { id (= room_id), game_id, phase, host, players, head, updated_at }

use super::{Event, EventLog, LeaseStore, MetaIndex, RoomMeta, RoomSnapshot};
use crate::db::Database;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

#[derive(Deserialize, SurrealValue)]
struct LeaseRow {
    #[allow(dead_code)]
    owner: String,
}

#[derive(Clone)]
pub struct SurrealStorage {
    db: Database,
}

impl SurrealStorage {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn record_key_to_string(rid: &RecordId) -> String {
    match &rid.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        // Crash early: any other key kind in our schema is a data
        // bug, and a `Debug` fallback would silently mangle IDs.
        other => panic!("unsupported RecordIdKey variant: {other:?}"),
    }
}

#[async_trait]
impl LeaseStore for SurrealStorage {
    async fn acquire(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool> {
        // Single-statement atomic UPSERT: SurrealDB locks the record key for
        // the duration of the statement, so two concurrent callers cannot
        // both win the lease. WHERE limits updates to: same owner (renew),
        // expired holder, or no row (insert path ignores WHERE). Empty
        // result => existing live lease held by someone else.
        let ttl_secs = ttl.as_secs() as i64;
        let q = r#"
            UPSERT type::record("room_lease", $rid)
            SET owner = $owner,
                expires = time::now() + duration::from_secs($ttl)
            WHERE owner = $owner OR expires < time::now();
        "#;
        let mut resp = self
            .db
            .query(q)
            .bind(("rid", room.to_string()))
            .bind(("owner", owner.to_string()))
            .bind(("ttl", ttl_secs))
            .await?;
        let rows: Vec<LeaseRow> = resp.take(0)?;
        Ok(!rows.is_empty())
    }

    async fn renew(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool> {
        let ttl_secs = ttl.as_secs() as i64;
        let q = r#"
            RETURN {
                UPDATE type::record("room_lease", $rid)
                SET expires = time::now() + duration::from_secs($ttl)
                WHERE owner = $owner;
                LET $rows = (SELECT * FROM type::record("room_lease", $rid));
                LET $row = $rows[0];
                RETURN $row IS NOT NONE AND $row.owner == $owner;
            };
        "#;
        let mut resp = self
            .db
            .query(q)
            .bind(("rid", room.to_string()))
            .bind(("owner", owner.to_string()))
            .bind(("ttl", ttl_secs))
            .await?;
        let ok: Option<bool> = resp.take(0)?;
        Ok(ok.unwrap_or(false))
    }

    async fn release(&self, room: &str, owner: &str) -> Result<()> {
        let q = r#"
            DELETE type::record("room_lease", $rid)
            WHERE owner = $owner;
        "#;
        self.db
            .query(q)
            .bind(("rid", room.to_string()))
            .bind(("owner", owner.to_string()))
            .await?;
        Ok(())
    }

    async fn rooms_owned_by(&self, owner: &str) -> Result<Vec<String>> {
        #[derive(Deserialize, SurrealValue)]
        struct Row {
            id: surrealdb::types::RecordId,
        }
        let q = r#"
            SELECT id FROM room_lease WHERE owner = $owner;
        "#;
        let mut resp = self.db.query(q).bind(("owner", owner.to_string())).await?;
        let rows: Vec<Row> = resp.take(0)?;
        Ok(rows.into_iter().map(|r| record_key_to_string(&r.id)).collect())
    }
}

#[async_trait]
impl EventLog for SurrealStorage {
    async fn append(&self, room: &str, owner: &str, kind: &str, payload: &str) -> Result<Event> {
        let q = r#"
            RETURN {
                LET $leases = (SELECT * FROM type::record("room_lease", $rid));
                LET $lease = $leases[0];
                IF $lease IS NONE OR $lease.owner != $owner OR $lease.expires < time::now() {
                    THROW "lease_lost";
                };
                LET $maxes = (SELECT VALUE seq FROM room_event WHERE room = $rid ORDER BY seq DESC LIMIT 1);
                LET $cur = $maxes[0] ?? 0;
                LET $next = $cur + 1;
                CREATE room_event CONTENT {
                    room: $rid,
                    seq: $next,
                    kind: $kind,
                    payload: $payload,
                    ts: time::now()
                };
                RETURN $next;
            };
        "#;
        let mut resp = self
            .db
            .query(q)
            .bind(("rid", room.to_string()))
            .bind(("owner", owner.to_string()))
            .bind(("kind", kind.to_string()))
            .bind(("payload", payload.to_string()))
            .await
            .map_err(|e| anyhow!("event append failed: {e}"))?;
        let seq: Option<u64> = resp.take(0)?;
        let seq = seq.ok_or_else(|| anyhow!("append returned no seq"))?;
        Ok(Event {
            seq,
            kind: kind.to_string(),
            payload: payload.to_string(),
        })
    }

    async fn read_since(&self, room: &str, since: u64) -> Result<Vec<Event>> {
        #[derive(Deserialize, SurrealValue)]
        struct Row {
            seq: u64,
            kind: String,
            payload: String,
        }
        let q = r#"
            SELECT seq, kind, payload
            FROM room_event
            WHERE room = $rid AND seq > $since
            ORDER BY seq ASC;
        "#;
        let mut resp = self
            .db
            .query(q)
            .bind(("rid", room.to_string()))
            .bind(("since", since as i64))
            .await?;
        let rows: Vec<Row> = resp.take(0)?;
        Ok(rows
            .into_iter()
            .map(|r| Event {
                seq: r.seq,
                kind: r.kind,
                payload: r.payload,
            })
            .collect())
    }

    async fn head(&self, room: &str) -> Result<u64> {
        let q = r#"
            SELECT VALUE seq
            FROM room_event
            WHERE room = $rid
            ORDER BY seq DESC
            LIMIT 1;
        "#;
        let mut resp = self.db.query(q).bind(("rid", room.to_string())).await?;
        let rows: Vec<u64> = resp.take(0)?;
        Ok(rows.into_iter().next().unwrap_or(0))
    }
}

#[async_trait]
impl MetaIndex for SurrealStorage {
    async fn upsert(
        &self,
        room: &str,
        game_id: &str,
        snapshot: &RoomSnapshot,
        head: u64,
    ) -> Result<()> {
        let q = r#"
            UPSERT type::record("room_meta", $rid)
            CONTENT {
                game_id: $game_id,
                phase: $phase,
                host: $host,
                players: $players,
                head: $head,
                updated_at: time::now()
            };
        "#;
        self.db
            .query(q)
            .bind(("rid", room.to_string()))
            .bind(("game_id", game_id.to_string()))
            .bind(("phase", snapshot.phase.clone()))
            .bind(("host", snapshot.host.clone()))
            .bind(("players", snapshot.players.clone()))
            .bind(("head", head as i64))
            .await?;
        Ok(())
    }

    async fn list(
        &self,
        game_id: Option<&str>,
        phase: Option<&str>,
        limit: u32,
    ) -> Result<Vec<RoomMeta>> {
        let mut filters: Vec<&str> = Vec::new();
        if game_id.is_some() {
            filters.push("game_id = $game_id");
        }
        if phase.is_some() {
            filters.push("phase = $phase");
        }
        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        };
        let q = format!(
            "SELECT id, game_id, phase, host, players, head, updated_at
             FROM room_meta{where_clause}
             ORDER BY updated_at DESC LIMIT $limit;"
        );
        let mut req = self.db.query(&q).bind(("limit", limit as i64));
        if let Some(g) = game_id {
            req = req.bind(("game_id", g.to_string()));
        }
        if let Some(p) = phase {
            req = req.bind(("phase", p.to_string()));
        }
        let mut resp = req.await?;
        let rows: Vec<RoomMetaRow> = resp.take(0)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Deserialize, SurrealValue)]
struct RoomMetaRow {
    id: RecordId,
    game_id: String,
    phase: String,
    host: Option<String>,
    players: Vec<String>,
    head: u64,
}

impl From<RoomMetaRow> for RoomMeta {
    fn from(r: RoomMetaRow) -> Self {
        Self {
            id: record_key_to_string(&r.id),
            game_id: r.game_id,
            phase: r.phase,
            host: r.host,
            players: r.players,
            head: r.head,
        }
    }
}
