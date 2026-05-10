use serde::Serialize;

/// Lowercase variant tag of a serde-serialised enum, e.g.
/// `SubmissionStatus::Pending` → `"pending"`. Strips the JSON quoting so
/// the result can be bound directly into a SurrealQL string slot.
pub fn enum_tag<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .expect("enum tag serialisation is infallible")
        .trim_matches('"')
        .to_string()
}
