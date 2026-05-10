// Chess playground bot: deterministic, picks the lexicographically first
// legal move in board-iteration order. Promotes to queen. Goal is replay
// determinism, not strength — same shape as the TTT and PD playground bots.

use crate::games::chess_logic::{
    legal_moves_for_turn, square_str, Color, MoveFlag, PieceType, State,
};
use crate::services::playground::PlaygroundStrategy;
use crate::services::room::logic::RoomLogic;

#[derive(Default)]
pub struct ChessStrategy {
    state: State,
    bot_color: Option<Color>,
    players: Vec<String>,
}

impl PlaygroundStrategy for ChessStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        // Mirror the room state by folding every event into our local copy.
        // The bot then queries `legal_moves_for_turn` for its own move
        // selection, matching whatever the validator would accept.
        crate::games::chess_logic::Chess::fold(&mut self.state, kind, payload);

        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !self.players.contains(&pid) {
                    self.players.push(pid);
                }
                None
            }
            "GAME_STARTED" => {
                self.bot_color = self
                    .players
                    .iter()
                    .position(|p| p == bot_pid)
                    .map(|i| if i == 0 { Color::White } else { Color::Black });
                self.maybe_move()
            }
            "MOVE" => self.maybe_move(),
            _ => None,
        }
    }
}

impl ChessStrategy {
    fn maybe_move(&self) -> Option<(String, String)> {
        let bot_color = self.bot_color?;
        if self.state.turn != bot_color {
            return None;
        }
        let legals = legal_moves_for_turn(&self.state);
        if legals.is_empty() {
            return None;
        }
        // Board-iteration order already gives a1→h1, a2→h2, ... a8→h8 for
        // white (the from-square index increases). For black we reverse the
        // from-square ordering so its first piece is closest to its own
        // back rank — pure aesthetic mirror, still deterministic.
        let mut sorted = legals;
        sorted.sort_by_key(|(f, t, fl)| {
            let from_key = if bot_color == Color::White { *f as i32 } else { -(*f as i32) };
            (from_key, *t, flag_order(*fl))
        });
        let (f, t, fl) = sorted.into_iter().next()?;
        let payload = match fl {
            MoveFlag::Promote(_) => format!("{} {} q", square_str(f), square_str(t)),
            _ => format!("{} {}", square_str(f), square_str(t)),
        };
        Some(("MOVE".into(), payload))
    }
}

// Promotion ordering: queen first so the bot always promotes to queen
// when a pawn reaches the last rank. Pawn / King in `Promote` are
// unreachable (the generator never produces them) but enumerated so the
// match stays exhaustive without a wildcard.
fn flag_order(fl: MoveFlag) -> u8 {
    match fl {
        MoveFlag::Promote(PieceType::Queen) => 0,
        MoveFlag::Promote(PieceType::Rook) => 1,
        MoveFlag::Promote(PieceType::Bishop) => 2,
        MoveFlag::Promote(PieceType::Knight) => 3,
        MoveFlag::Promote(PieceType::Pawn) | MoveFlag::Promote(PieceType::King) => 0,
        MoveFlag::CastleK => 4,
        MoveFlag::CastleQ => 5,
        MoveFlag::EnPassant => 6,
        MoveFlag::Normal => 7,
    }
}
