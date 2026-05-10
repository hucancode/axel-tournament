// Xiangqi playground bot. Deterministic, not strategic: enumerates own
// pieces in board order and plays the lexicographically first
// "<from> <to>" legal move. Mirrors the shape of `ttt_playground.rs`.

use crate::games::xiangqi_logic::{legal_moves, Kind, Piece};
use crate::services::playground::PlaygroundStrategy;

const COLS: usize = 9;
const ROWS: usize = 10;
const CELLS: usize = COLS * ROWS;

pub struct XiangqiStrategy {
    board: [Option<Piece>; CELLS],
    bot_side: Option<u8>,
    players: Vec<String>,
    started: bool,
}

impl Default for XiangqiStrategy {
    fn default() -> Self {
        Self {
            board: [None; CELLS],
            bot_side: None,
            players: Vec::new(),
            started: false,
        }
    }
}

impl PlaygroundStrategy for XiangqiStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !self.players.contains(&pid) {
                    self.players.push(pid);
                }
                None
            }
            "GAME_STARTED" => {
                self.board = initial_board();
                self.started = true;
                self.bot_side = self.players.iter().position(|p| p == bot_pid).map(|i| i as u8);
                if self.bot_side == Some(0) {
                    self.first_legal_move()
                } else {
                    None
                }
            }
            "MOVE" => {
                let mut it = payload.split_whitespace();
                let pid = it.next()?;
                let from = it.next()?;
                let to = it.next()?;
                let (fr, fc) = parse_square(from)?;
                let (tr, tc) = parse_square(to)?;
                let p = self.board[idx(fr, fc)]?;
                self.board[idx(fr, fc)] = None;
                self.board[idx(tr, tc)] = Some(p);

                if pid != bot_pid {
                    self.first_legal_move()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl XiangqiStrategy {
    fn first_legal_move(&self) -> Option<(String, String)> {
        if !self.started {
            return None;
        }
        let side = self.bot_side?;
        let moves = legal_moves(&self.board, side);
        // legal_moves already enumerates in (from_row, from_col, to_row,
        // to_col) order, which is exactly the lex order on the
        // "<file><rank> <file><rank>" payload.
        let (fr, fc, tr, tc) = *moves.first()?;
        Some((
            "MOVE".into(),
            format!(
                "{} {}",
                square_to_str(fr, fc),
                square_to_str(tr, tc)
            ),
        ))
    }
}

#[inline]
fn idx(row: usize, col: usize) -> usize {
    row * COLS + col
}

fn parse_square(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    if bytes.len() < 2 || bytes.len() > 3 {
        return None;
    }
    let file = bytes[0];
    if !(b'a'..=b'i').contains(&file) {
        return None;
    }
    let col = (file - b'a') as usize;
    let rank: usize = std::str::from_utf8(&bytes[1..]).ok()?.parse().ok()?;
    if rank >= ROWS {
        return None;
    }
    Some((rank, col))
}

fn square_to_str(row: usize, col: usize) -> String {
    let mut s = String::with_capacity(3);
    s.push((b'a' + col as u8) as char);
    s.push_str(&row.to_string());
    s
}

fn initial_board() -> [Option<Piece>; CELLS] {
    let mut b: [Option<Piece>; CELLS] = [None; CELLS];
    let back = [
        Kind::Rook,
        Kind::Horse,
        Kind::Elephant,
        Kind::Advisor,
        Kind::General,
        Kind::Advisor,
        Kind::Elephant,
        Kind::Horse,
        Kind::Rook,
    ];
    for (col, k) in back.iter().enumerate() {
        b[idx(0, col)] = Some(Piece { side: 0, kind: *k });
        b[idx(9, col)] = Some(Piece { side: 1, kind: *k });
    }
    for col in [1usize, 7] {
        b[idx(2, col)] = Some(Piece { side: 0, kind: Kind::Cannon });
        b[idx(7, col)] = Some(Piece { side: 1, kind: Kind::Cannon });
    }
    for col in [0usize, 2, 4, 6, 8] {
        b[idx(3, col)] = Some(Piece { side: 0, kind: Kind::Pawn });
        b[idx(6, col)] = Some(Piece { side: 1, kind: Kind::Pawn });
    }
    b
}
