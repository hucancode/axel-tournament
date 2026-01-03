use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GameMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub supported_languages: &'static [&'static str],
    pub rounds_per_match: u32,
    pub repetitions: u32,
    pub bot_timeout_ms: u32,
    pub human_timeout_ms: u32,
    pub cpu_limit: f64,
    pub bot_turn_timeout_ms: u64,
    pub human_turn_timeout_ms: u64,
    pub memory_limit_mb: u64,
}

// Static game registry matching the API server
pub static GAMES: &[GameMetadata] = &[
    GameMetadata {
        id: "rock-paper-scissors",
        name: "Rock Paper Scissors",
        description: "Classic rock-paper-scissors game for 2 players",
        supported_languages: &["rust", "go", "c"],
        rounds_per_match: 100,
        repetitions: 1,
        bot_timeout_ms: 5000,
        human_timeout_ms: 30000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 2000,
        human_turn_timeout_ms: 10000,
        memory_limit_mb: 64,
    },
    GameMetadata {
        id: "prisoners-dilemma",
        name: "Prisoner's Dilemma",
        description: "Classic game theory prisoner's dilemma",
        supported_languages: &["rust", "go", "c"],
        rounds_per_match: 100,
        repetitions: 1,
        bot_timeout_ms: 5000,
        human_timeout_ms: 30000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 2000,
        human_turn_timeout_ms: 10000,
        memory_limit_mb: 64,
    },
    GameMetadata {
        id: "tic-tac-toe",
        name: "Tic Tac Toe",
        description: "Classic tic-tac-toe game for 2 players",
        supported_languages: &["rust", "go", "c"],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 60000,
        human_timeout_ms: 120000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 30000,
        human_turn_timeout_ms: 60000,
        memory_limit_mb: 64,
    },
];

pub fn find_game_by_id(id: &str) -> Option<&'static GameMetadata> {
    GAMES.iter().find(|g| g.id == id)
}
