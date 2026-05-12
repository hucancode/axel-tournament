use surrealdb::types::{RecordId, RecordIdKey, ToSql};

/// Bare key part of a record id ("alice" from `user:alice`). Falls
/// back to full SQL form for non-string keys (uuid, number, etc).
pub fn bare_key(r: &RecordId) -> String {
    match &r.key {
        RecordIdKey::String(s) => s.clone(),
        _ => r.to_sql(),
    }
}

pub fn opt_bare_key(r: &Option<RecordId>) -> Option<String> {
    r.as_ref().map(bare_key)
}

pub fn vec_bare_key(rs: &[RecordId]) -> Vec<String> {
    rs.iter().map(bare_key).collect()
}

/// Build a record id from a known table + bare key string. Use at the
/// HTTP boundary to lift a frontend-supplied key into the internal
/// `RecordId` form.
pub fn rid(table: &'static str, key: impl Into<String>) -> RecordId {
    RecordId::new(table, key.into())
}
