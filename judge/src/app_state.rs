use crate::db::Database;
use crate::services::capacity::CapacityTracker;
use crate::services::storage::MetaIndex;
use std::sync::Arc;

/// Shared judge state used by HTTP middleware, capacity, and discovery
/// endpoints. Per-game v2 room registries live in `WsContext<L>`
/// (services::room::ws).
pub struct AppState {
    pub db: Database,
    pub capacity: CapacityTracker,
    pub jwt_secret: String,
    pub meta: Arc<dyn MetaIndex>,
}
