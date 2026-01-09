use std::sync::Arc;

use crate::services::capacity::CapacityTracker;
use crate::db::Database;
use crate::models::game::Game;
use crate::services::room::RoomManager;

pub struct AppState<G: Game> {
    pub db: Database,
    pub game: G,
    pub capacity: CapacityTracker,
    pub room_manager: Arc<RoomManager>,
    pub jwt_secret: String,
}
