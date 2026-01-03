use crate::capacity::CapacityTracker;
use crate::db::Database;
use crate::game_logic::GameLogic;
use crate::room::RoomManager;

pub struct AppState<G: GameLogic> {
    #[allow(dead_code)]
    pub db: Database,
    pub game: G,
    pub capacity: CapacityTracker,
    pub room_manager: RoomManager,
}
