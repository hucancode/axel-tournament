use serde::Serialize;

/// How a finished match contributes to the leaderboard.
/// `Elo` = 1v1 win/lose ELO update.
/// `Score` = accumulate per-player score (e.g. chips for poker, rounds won
/// for RPS/PD) into the participant's `elo` column. Score-based games also
/// use a different matchmaker (continuous score-pairing rather than bracket).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScoringKind {
    Elo,
    Score,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
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
    pub game_rounds: Option<u32>,
    pub game_round_variance: Option<u32>,
    pub scoring_kind: ScoringKind,
}

// Static game registry matching the API server
pub static GAMES: &[Metadata] = &[
    Metadata {
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
        game_rounds: Some(5),
        game_round_variance: Some(2),
        scoring_kind: ScoringKind::Score,
    },
    Metadata {
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
        game_rounds: Some(10),
        game_round_variance: Some(3),
        scoring_kind: ScoringKind::Score,
    },
    Metadata {
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
        game_rounds: None,
        game_round_variance: None,
        scoring_kind: ScoringKind::Elo,
    },
    Metadata {
        id: "chess",
        name: "Chess",
        description: "Standard chess for 2 players",
        supported_languages: &["rust", "go", "c"],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 600000,
        human_timeout_ms: 1800000,
        cpu_limit: 2.0,
        bot_turn_timeout_ms: 60000,
        human_turn_timeout_ms: 120000,
        memory_limit_mb: 128,
        game_rounds: None,
        game_round_variance: None,
        scoring_kind: ScoringKind::Elo,
    },
    Metadata {
        id: "xiangqi",
        name: "Xiangqi",
        description: "Chinese chess for 2 players",
        supported_languages: &["rust", "go", "c"],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 600000,
        human_timeout_ms: 1800000,
        cpu_limit: 2.0,
        bot_turn_timeout_ms: 60000,
        human_turn_timeout_ms: 120000,
        memory_limit_mb: 128,
        game_rounds: None,
        game_round_variance: None,
        scoring_kind: ScoringKind::Elo,
    },
    Metadata {
        id: "poker",
        name: "Poker",
        description: "Heads-up no-limit Texas Hold'em",
        supported_languages: &["rust", "go", "c"],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 600000,
        human_timeout_ms: 1800000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 15000,
        human_turn_timeout_ms: 60000,
        memory_limit_mb: 128,
        game_rounds: Some(10),
        game_round_variance: None,
        scoring_kind: ScoringKind::Score,
    },
];

pub fn find_game_by_id(id: &str) -> Option<&'static Metadata> {
    GAMES.iter().find(|g| g.id == id)
}
