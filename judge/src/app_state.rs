use std::sync::Arc;

use crate::capacity::CapacityTracker;
use crate::db::Database;
use crate::games::Game;
use crate::room::RoomManager;

pub struct AppState<G: Game> {
    #[allow(dead_code)]
    pub db: Database,
    pub game: G,
    pub capacity: CapacityTracker,
    pub room_manager: Arc<RoomManager>,
    pub jwt_secret: String,
}
