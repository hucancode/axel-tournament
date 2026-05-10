// Xiangqi (Chinese chess) as RoomLogic.
//
// Spec: judge/protocols/xiangqi.md.
// 9 cols (files a..i) by 10 rows (ranks 0..9).
// Player 0 = red (rows 0..4). Player 1 = black (rows 5..9). Red moves first.
// Single source of truth: event log; `fold` must be total.

use crate::services::room::logic::RoomLogic;
use crate::services::storage::RoomSnapshot;

const MAX_PLAYERS: usize = 2;
const COLS: usize = 9;
const ROWS: usize = 10;
const CELLS: usize = COLS * ROWS;

/// Piece kinds. Encoded compactly so the board array is `Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    General,
    Advisor,
    Elephant,
    Horse,
    Rook,
    Cannon,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub side: u8, // 0 = red, 1 = black
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    Playing,
    Finished,
}

#[derive(Debug, Clone)]
pub struct State {
    pub phase: Phase,
    pub players: Vec<String>,
    pub host: Option<String>,
    pub board: [Option<Piece>; CELLS],
    pub turn: u8, // 0 = red to move, 1 = black to move
    pub winner: Option<u8>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            board: [None; CELLS],
            turn: 0,
            winner: None,
        }
    }
}

impl State {
    fn player_index(&self, player_id: &str) -> Option<u8> {
        self.players.iter().position(|p| p == player_id).map(|i| i as u8)
    }
}

pub struct Xiangqi;

// -------------------- coordinate helpers --------------------

#[inline]
fn idx(row: usize, col: usize) -> usize {
    row * COLS + col
}

#[inline]
fn in_board(row: i32, col: i32) -> bool {
    row >= 0 && row < ROWS as i32 && col >= 0 && col < COLS as i32
}

/// Parse `<file><rank>`, e.g. "e0" -> (row=0, col=4). Files a..i, ranks 0..9.
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

// -------------------- board setup --------------------

fn initial_board() -> [Option<Piece>; CELLS] {
    let mut b: [Option<Piece>; CELLS] = [None; CELLS];

    // Red back rank (row 0): R H E A K A E H R.
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

    // Cannons: red row 2 cols 1 & 7; black row 7 cols 1 & 7.
    for col in [1usize, 7] {
        b[idx(2, col)] = Some(Piece { side: 0, kind: Kind::Cannon });
        b[idx(7, col)] = Some(Piece { side: 1, kind: Kind::Cannon });
    }

    // Pawns: red row 3 cols 0,2,4,6,8; black row 6 cols 0,2,4,6,8.
    for col in [0usize, 2, 4, 6, 8] {
        b[idx(3, col)] = Some(Piece { side: 0, kind: Kind::Pawn });
        b[idx(6, col)] = Some(Piece { side: 1, kind: Kind::Pawn });
    }

    b
}

// -------------------- palace / river helpers --------------------

#[inline]
fn in_palace(side: u8, row: i32, col: i32) -> bool {
    if !(col >= 3 && col <= 5) {
        return false;
    }
    if side == 0 {
        row >= 0 && row <= 2
    } else {
        row >= 7 && row <= 9
    }
}

#[inline]
fn crossed_river(side: u8, row: i32) -> bool {
    if side == 0 {
        row >= 5
    } else {
        row <= 4
    }
}

#[inline]
fn on_own_side(side: u8, row: i32) -> bool {
    if side == 0 {
        row <= 4
    } else {
        row >= 5
    }
}

// -------------------- move generation (pseudo-legal) --------------------

/// Generate pseudo-legal moves for the piece at `(from_row, from_col)`.
/// Pseudo-legal = follows piece movement rules, but does not yet check
/// king-in-check or flying-general. Caller must filter.
fn pseudo_moves(board: &[Option<Piece>; CELLS], from_row: usize, from_col: usize) -> Vec<(usize, usize)> {
    let Some(piece) = board[idx(from_row, from_col)] else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let r = from_row as i32;
    let c = from_col as i32;

    let push = |out: &mut Vec<(usize, usize)>, nr: i32, nc: i32| {
        if !in_board(nr, nc) {
            return;
        }
        // Cannot capture own piece.
        if let Some(t) = board[idx(nr as usize, nc as usize)] {
            if t.side == piece.side {
                return;
            }
        }
        out.push((nr as usize, nc as usize));
    };

    match piece.kind {
        Kind::General => {
            for (dr, dc) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nr = r + dr;
                let nc = c + dc;
                if in_palace(piece.side, nr, nc) {
                    push(&mut out, nr, nc);
                }
            }
        }
        Kind::Advisor => {
            for (dr, dc) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let nr = r + dr;
                let nc = c + dc;
                if in_palace(piece.side, nr, nc) {
                    push(&mut out, nr, nc);
                }
            }
        }
        Kind::Elephant => {
            for (dr, dc) in [(2, 2), (2, -2), (-2, 2), (-2, -2)] {
                let nr = r + dr;
                let nc = c + dc;
                if !in_board(nr, nc) {
                    continue;
                }
                if !on_own_side(piece.side, nr) {
                    continue; // cannot cross river
                }
                let mr = r + dr / 2;
                let mc = c + dc / 2;
                if board[idx(mr as usize, mc as usize)].is_some() {
                    continue; // blocked midpoint
                }
                push(&mut out, nr, nc);
            }
        }
        Kind::Horse => {
            // 8 L-moves; each blocked by the orthogonally adjacent leg.
            let jumps: [(i32, i32, i32, i32); 8] = [
                (2, 1, 1, 0),
                (2, -1, 1, 0),
                (-2, 1, -1, 0),
                (-2, -1, -1, 0),
                (1, 2, 0, 1),
                (-1, 2, 0, 1),
                (1, -2, 0, -1),
                (-1, -2, 0, -1),
            ];
            for (dr, dc, lr, lc) in jumps {
                let nr = r + dr;
                let nc = c + dc;
                if !in_board(nr, nc) {
                    continue;
                }
                let leg_r = r + lr;
                let leg_c = c + lc;
                if board[idx(leg_r as usize, leg_c as usize)].is_some() {
                    continue;
                }
                push(&mut out, nr, nc);
            }
        }
        Kind::Rook => {
            for (dr, dc) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let mut nr = r + dr;
                let mut nc = c + dc;
                while in_board(nr, nc) {
                    match board[idx(nr as usize, nc as usize)] {
                        None => out.push((nr as usize, nc as usize)),
                        Some(t) => {
                            if t.side != piece.side {
                                out.push((nr as usize, nc as usize));
                            }
                            break;
                        }
                    }
                    nr += dr;
                    nc += dc;
                }
            }
        }
        Kind::Cannon => {
            for (dr, dc) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                // Phase 1: slide while empty (non-capture moves).
                let mut nr = r + dr;
                let mut nc = c + dc;
                while in_board(nr, nc) && board[idx(nr as usize, nc as usize)].is_none() {
                    out.push((nr as usize, nc as usize));
                    nr += dr;
                    nc += dc;
                }
                // Phase 2: we hit a piece (the screen). Look past it for
                // exactly one piece to capture.
                if !in_board(nr, nc) {
                    continue;
                }
                nr += dr;
                nc += dc;
                while in_board(nr, nc) {
                    if let Some(t) = board[idx(nr as usize, nc as usize)] {
                        if t.side != piece.side {
                            out.push((nr as usize, nc as usize));
                        }
                        break;
                    }
                    nr += dr;
                    nc += dc;
                }
            }
        }
        Kind::Pawn => {
            let forward: i32 = if piece.side == 0 { 1 } else { -1 };
            // forward 1
            push(&mut out, r + forward, c);
            // sideways after crossing river
            if crossed_river(piece.side, r) {
                push(&mut out, r, c + 1);
                push(&mut out, r, c - 1);
            }
        }
    }

    out
}

// -------------------- check / flying general --------------------

fn find_general(board: &[Option<Piece>; CELLS], side: u8) -> Option<(usize, usize)> {
    for row in 0..ROWS {
        for col in 0..COLS {
            if let Some(p) = board[idx(row, col)] {
                if p.side == side && p.kind == Kind::General {
                    return Some((row, col));
                }
            }
        }
    }
    None
}

/// True iff the two generals share a file with nothing between them.
fn generals_face(board: &[Option<Piece>; CELLS]) -> bool {
    let (Some((r0, c0)), Some((r1, c1))) = (find_general(board, 0), find_general(board, 1)) else {
        return false;
    };
    if c0 != c1 {
        return false;
    }
    let (lo, hi) = if r0 < r1 { (r0, r1) } else { (r1, r0) };
    for r in (lo + 1)..hi {
        if board[idx(r, c0)].is_some() {
            return false;
        }
    }
    true
}

/// True iff `side`'s general is attacked by any opposing piece.
fn in_check(board: &[Option<Piece>; CELLS], side: u8) -> bool {
    let Some((kr, kc)) = find_general(board, side) else {
        // No general means already lost; for "in check" purposes treat as
        // not-in-check so callers don't double-report.
        return false;
    };
    let opp = 1 - side;
    for row in 0..ROWS {
        for col in 0..COLS {
            let Some(p) = board[idx(row, col)] else { continue };
            if p.side != opp {
                continue;
            }
            for (tr, tc) in pseudo_moves(board, row, col) {
                if tr == kr && tc == kc {
                    return true;
                }
            }
        }
    }
    false
}

/// Apply a move on a copy of the board, returning the resulting board.
fn after_move(board: &[Option<Piece>; CELLS], fr: usize, fc: usize, tr: usize, tc: usize) -> [Option<Piece>; CELLS] {
    let mut next = *board;
    let p = next[idx(fr, fc)];
    next[idx(fr, fc)] = None;
    next[idx(tr, tc)] = p;
    next
}

/// Enumerate every legal move for `side`. Returns (from_row, from_col,
/// to_row, to_col). Order is deterministic: by (from_row, from_col,
/// to_row, to_col).
pub fn legal_moves(board: &[Option<Piece>; CELLS], side: u8) -> Vec<(usize, usize, usize, usize)> {
    let mut out = Vec::new();
    for fr in 0..ROWS {
        for fc in 0..COLS {
            let Some(p) = board[idx(fr, fc)] else { continue };
            if p.side != side {
                continue;
            }
            for (tr, tc) in pseudo_moves(board, fr, fc) {
                let next = after_move(board, fr, fc, tr, tc);
                if in_check(&next, side) {
                    continue;
                }
                if generals_face(&next) {
                    continue;
                }
                out.push((fr, fc, tr, tc));
            }
        }
    }
    out
}

fn opposing_general_captured(captured: Option<Piece>, mover_side: u8) -> bool {
    match captured {
        Some(p) => p.side != mover_side && p.kind == Kind::General,
        None => false,
    }
}

// -------------------- RoomLogic impl --------------------

impl RoomLogic for Xiangqi {
    type State = State;

    fn fold(state: &mut Self::State, kind: &str, payload: &str) {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !state.players.contains(&pid) && state.players.len() < MAX_PLAYERS {
                    state.players.push(pid.clone());
                    if state.host.is_none() {
                        state.host = Some(pid);
                    }
                }
            }
            "PLAYER_LEFT" => {
                let pid = payload.trim();
                if let Some(idx) = state.players.iter().position(|p| p == pid) {
                    state.players.remove(idx);
                }
                if state.host.as_deref() == Some(pid) {
                    state.host = state.players.first().cloned();
                }
            }
            "HOST_CHANGED" => {
                state.host = Some(payload.trim().to_string());
            }
            "GAME_STARTED" => {
                state.phase = Phase::Playing;
                state.board = initial_board();
                state.turn = 0;
                state.winner = None;
            }
            "MOVE" => {
                // payload: "<pid> <from> <to>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() != 3 {
                    return;
                }
                let pid = parts[0];
                let Some((fr, fc)) = parse_square(parts[1]) else { return };
                let Some((tr, tc)) = parse_square(parts[2]) else { return };
                let Some(side) = state.player_index(pid) else { return };
                let Some(piece) = state.board[idx(fr, fc)] else { return };
                if piece.side != side {
                    return;
                }
                state.board[idx(fr, fc)] = None;
                state.board[idx(tr, tc)] = Some(piece);
                state.turn = 1 - side;
            }
            "WINNER" => {
                if let Ok(w) = payload.trim().parse::<u8>() {
                    state.winner = Some(w);
                    state.phase = Phase::Finished;
                }
            }
            "CHAT" => {}
            _ => {}
        }
    }

    fn validate(
        state: &Self::State,
        player: &str,
        kind: &str,
        payload: &str,
    ) -> Result<Vec<(String, String)>, String> {
        match kind {
            "JOIN" => {
                if state.players.iter().any(|p| p == player) {
                    return Ok(Vec::new());
                }
                if state.phase != Phase::Lobby {
                    return Err("game already started".into());
                }
                if state.players.len() >= MAX_PLAYERS {
                    return Err("room full".into());
                }
                Ok(vec![("PLAYER_JOINED".into(), player.to_string())])
            }
            "LEAVE" => {
                if !state.players.iter().any(|p| p == player) {
                    return Ok(Vec::new());
                }
                let mut out = vec![("PLAYER_LEFT".into(), player.to_string())];
                if state.host.as_deref() == Some(player) {
                    if let Some(h) = state.players.iter().find(|p| p.as_str() != player).cloned() {
                        out.push(("HOST_CHANGED".into(), h));
                    }
                }
                Ok(out)
            }
            "CHAT" => {
                let msg = sanitize(payload, 500);
                if msg.is_empty() {
                    return Ok(Vec::new());
                }
                Ok(vec![("CHAT".into(), format!("{player} {msg}"))])
            }
            "START" => {
                if state.host.as_deref() != Some(player) {
                    return Err("only host may start".into());
                }
                if state.phase != Phase::Lobby {
                    return Err("already started".into());
                }
                if state.players.len() < 2 {
                    return Err("need 2 players".into());
                }
                Ok(vec![("GAME_STARTED".into(), String::new())])
            }
            "MOVE" => {
                if state.phase != Phase::Playing {
                    return Err("not playing".into());
                }
                let side = state
                    .player_index(player)
                    .ok_or_else(|| "not in room".to_string())?;
                if state.turn != side {
                    return Err("not your turn".into());
                }

                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() != 2 {
                    return Err("expected: MOVE <from> <to>".into());
                }
                let (fr, fc) = parse_square(parts[0]).ok_or_else(|| "bad from".to_string())?;
                let (tr, tc) = parse_square(parts[1]).ok_or_else(|| "bad to".to_string())?;
                if (fr, fc) == (tr, tc) {
                    return Err("from == to".into());
                }
                let piece = state.board[idx(fr, fc)].ok_or_else(|| "no piece at from".to_string())?;
                if piece.side != side {
                    return Err("not your piece".into());
                }
                if !pseudo_moves(&state.board, fr, fc)
                    .iter()
                    .any(|&(r, c)| r == tr && c == tc)
                {
                    return Err("illegal move for piece".into());
                }
                let next = after_move(&state.board, fr, fc, tr, tc);
                if in_check(&next, side) {
                    return Err("would leave own general in check".into());
                }
                if generals_face(&next) {
                    return Err("flying general".into());
                }

                let captured = state.board[idx(tr, tc)];
                let from_s = square_to_str(fr, fc);
                let to_s = square_to_str(tr, tc);
                let mut out = vec![(
                    "MOVE".into(),
                    format!("{player} {from_s} {to_s}"),
                )];

                if opposing_general_captured(captured, side) {
                    out.push(("WINNER".into(), side.to_string()));
                    return Ok(out);
                }

                // Stalemate check: opponent has zero legal moves -> they lose.
                let opp = 1 - side;
                if legal_moves(&next, opp).is_empty() {
                    out.push(("WINNER".into(), side.to_string()));
                }
                Ok(out)
            }
            _ => Err(format!("unknown action: {kind}")),
        }
    }

    fn max_players() -> usize {
        MAX_PLAYERS
    }

    fn game_id() -> &'static str {
        "xiangqi"
    }

    fn pending_players(state: &Self::State) -> Vec<String> {
        if state.phase != Phase::Playing {
            return Vec::new();
        }
        state
            .players
            .get(state.turn as usize)
            .cloned()
            .into_iter()
            .collect()
    }

    fn snapshot(state: &Self::State) -> RoomSnapshot {
        RoomSnapshot {
            phase: phase_str(&state.phase),
            host: state.host.clone(),
            players: state.players.clone(),
        }
    }
}

fn phase_str(phase: &Phase) -> String {
    match phase {
        Phase::Lobby => "lobby",
        Phase::Playing => "playing",
        Phase::Finished => "finished",
    }
    .to_string()
}

fn sanitize(s: &str, max_len: usize) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .take(max_len)
        .collect();
    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(s: &mut State, kind: &str, payload: &str) {
        Xiangqi::fold(s, kind, payload);
    }

    fn drive(s: &mut State, player: &str, kind: &str, payload: &str) {
        let evs = Xiangqi::validate(s, player, kind, payload).expect("validate ok");
        for (k, p) in &evs {
            apply(s, k, p);
        }
    }

    fn place(b: &mut [Option<Piece>; CELLS], side: u8, kind: Kind, sq: &str) {
        let (r, c) = parse_square(sq).expect("valid square");
        b[idx(r, c)] = Some(Piece { side, kind });
    }

    fn empty_board_with_kings() -> [Option<Piece>; CELLS] {
        let mut b: [Option<Piece>; CELLS] = [None; CELLS];
        // Park each general in its own palace, off the e-file so we don't
        // accidentally trigger the flying-general check in scenario tests.
        b[idx(0, 3)] = Some(Piece { side: 0, kind: Kind::General });
        b[idx(9, 5)] = Some(Piece { side: 1, kind: Kind::General });
        b
    }

    fn started_game() -> State {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        s
    }

    // -------- coordinate parser --------

    #[test]
    fn parse_square_round_trip() {
        for sq in ["a0", "i9", "e5", "c2"] {
            let (r, c) = parse_square(sq).unwrap();
            assert_eq!(square_to_str(r, c), sq);
        }
        assert!(parse_square("j0").is_none());
        assert!(parse_square("a10").is_none());
        assert!(parse_square("").is_none());
    }

    // -------- piece movement --------

    #[test]
    fn rook_captures_along_file() {
        let mut b = empty_board_with_kings();
        place(&mut b, 0, Kind::Rook, "e2");
        place(&mut b, 1, Kind::Pawn, "e7");
        let moves = pseudo_moves(&b, 2, 4);
        // Should include capture at e7 and all empty squares in between.
        assert!(moves.contains(&(7, 4)));
        assert!(moves.contains(&(3, 4)));
        assert!(moves.contains(&(6, 4)));
        // Should not include past the captured pawn.
        assert!(!moves.contains(&(8, 4)));
    }

    #[test]
    fn horse_blocked_by_leg() {
        // Red horse at b2 (row 2, col 1). Block leg at b3 (row 3, col 1).
        // Without block, horse can go to a4 (row 4, col 0) and c4 (row 4, col 2).
        let mut b = empty_board_with_kings();
        place(&mut b, 0, Kind::Horse, "b2");
        let unblocked = pseudo_moves(&b, 2, 1);
        assert!(unblocked.contains(&(4, 0)));
        assert!(unblocked.contains(&(4, 2)));

        place(&mut b, 0, Kind::Pawn, "b3");
        let blocked = pseudo_moves(&b, 2, 1);
        // The two "north" L-moves are blocked by the leg at b3.
        assert!(!blocked.contains(&(4, 0)));
        assert!(!blocked.contains(&(4, 2)));
        // But the two "south" moves (row 0) are still pseudo-legal.
        assert!(blocked.contains(&(0, 0)) || blocked.contains(&(0, 2)));
    }

    #[test]
    fn cannon_needs_screen_to_capture() {
        // Red cannon at e2. Black pawn at e7. No screen between.
        let mut b = empty_board_with_kings();
        place(&mut b, 0, Kind::Cannon, "e2");
        place(&mut b, 1, Kind::Pawn, "e7");
        let no_screen = pseudo_moves(&b, 2, 4);
        assert!(!no_screen.contains(&(7, 4)), "cannon should not capture without screen");
        // It can still slide to empty squares.
        assert!(no_screen.contains(&(3, 4)));

        // Add a screen at e5; now e7 is capturable.
        place(&mut b, 0, Kind::Pawn, "e5");
        let with_screen = pseudo_moves(&b, 2, 4);
        assert!(with_screen.contains(&(7, 4)), "cannon should capture over a screen");
        // Cannot slide through the screen any more.
        assert!(!with_screen.contains(&(5, 4)));
        assert!(!with_screen.contains(&(6, 4)));
    }

    #[test]
    fn elephant_cannot_cross_river() {
        // Red elephant at c4 trying to jump to e6 (across river).
        let mut b = empty_board_with_kings();
        place(&mut b, 0, Kind::Elephant, "c4");
        let moves = pseudo_moves(&b, 4, 2);
        // (6, 4) is across the river -> not allowed.
        assert!(!moves.contains(&(6, 4)));
        // (2, 0) and (2, 4) are on red side and should be allowed.
        assert!(moves.contains(&(2, 0)));
        assert!(moves.contains(&(2, 4)));
    }

    #[test]
    fn elephant_blocked_by_midpoint() {
        let mut b = empty_board_with_kings();
        place(&mut b, 0, Kind::Elephant, "c0");
        // Block midpoint at d1 (row 1, col 3).
        place(&mut b, 0, Kind::Pawn, "d1");
        let moves = pseudo_moves(&b, 0, 2);
        assert!(!moves.contains(&(2, 4)));
        // The other diagonal (a2) has no midpoint piece so it's still legal.
        assert!(moves.contains(&(2, 0)));
    }

    #[test]
    fn pawn_sideways_only_after_river() {
        let mut b = empty_board_with_kings();
        place(&mut b, 0, Kind::Pawn, "e3");
        let pre = pseudo_moves(&b, 3, 4);
        assert_eq!(pre, vec![(4, 4)]);

        // Move pawn across the river.
        let mut b2 = empty_board_with_kings();
        place(&mut b2, 0, Kind::Pawn, "e5");
        let post = pseudo_moves(&b2, 5, 4);
        assert!(post.contains(&(6, 4)));
        assert!(post.contains(&(5, 3)));
        assert!(post.contains(&(5, 5)));
    }

    // -------- flying general --------

    #[test]
    fn flying_general_rejected() {
        // Set up a position where moving exposes the general flying rule.
        let mut s = started_game();
        // Clear column e between the generals so any move that opens it triggers.
        // Easier: build a custom state.
        s.board = [None; CELLS];
        s.board[idx(0, 4)] = Some(Piece { side: 0, kind: Kind::General });
        s.board[idx(9, 4)] = Some(Piece { side: 1, kind: Kind::General });
        // Add a red pawn at e1 (row 1 col 4) that, if pushed sideways via a rook
        // analogy, would expose the file. Instead, place a red rook on e2 and
        // try to move it off file e.
        s.board[idx(2, 4)] = Some(Piece { side: 0, kind: Kind::Rook });
        s.turn = 0;

        // Moving the rook off the e-file leaves both generals facing.
        let res = Xiangqi::validate(&s, "a", "MOVE", "e2 d2");
        assert!(res.is_err(), "flying general should reject this move");
    }

    // -------- capture-the-general termination --------

    #[test]
    fn rook_capture_general_wins() {
        let mut s = started_game();
        // Empty board, general capture scenario. Red to move.
        s.board = [None; CELLS];
        s.board[idx(0, 3)] = Some(Piece { side: 0, kind: Kind::General });
        s.board[idx(9, 4)] = Some(Piece { side: 1, kind: Kind::General });
        // Red rook on e1 about to capture the black general on e9 (clear file).
        s.board[idx(1, 4)] = Some(Piece { side: 0, kind: Kind::Rook });
        s.turn = 0;

        drive(&mut s, "a", "MOVE", "e1 e9");
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, Some(0));
    }

    // -------- stalemate awards win to the other side --------

    #[test]
    fn stalemate_makes_other_side_winner() {
        // Build a position where after red's move, black has no legal
        // moves: black general boxed in by its own advisors, surrounded by
        // attacks. Simplest: bare black general in palace corner with all
        // four adjacent palace squares attacked.
        let mut s = started_game();
        s.board = [None; CELLS];
        // Black general at e9. Black has no other pieces.
        s.board[idx(9, 4)] = Some(Piece { side: 1, kind: Kind::General });
        // Red general parked somewhere safe, off-file from black.
        s.board[idx(0, 3)] = Some(Piece { side: 0, kind: Kind::General });
        // Red rooks attacking d9 and f9, plus one ready to attack e8.
        // Black general at e9 can move to d9, f9, or e8 only.
        // Cover d9 with a rook on d1; cover f9 with a rook on f1.
        s.board[idx(1, 3)] = Some(Piece { side: 0, kind: Kind::Rook });
        s.board[idx(1, 5)] = Some(Piece { side: 0, kind: Kind::Rook });
        // Red rook at a8 covers e8 once we slide it to e8.
        s.board[idx(8, 0)] = Some(Piece { side: 0, kind: Kind::Rook });
        s.turn = 0;

        // Slide a8-rook to e8 -> attacks e9 (capture-the-king sequence
        // would normally end it), so we instead cover e8 from a different
        // angle: place rook directly on e8 already and have red make a
        // tempo move that doesn't change attack picture.
        //
        // Re-stage: skip the moving-rook and set the position with red
        // rook already on e8, plus a spare red pawn we can push.
        s.board = [None; CELLS];
        s.board[idx(9, 4)] = Some(Piece { side: 1, kind: Kind::General });
        s.board[idx(0, 3)] = Some(Piece { side: 0, kind: Kind::General });
        s.board[idx(1, 3)] = Some(Piece { side: 0, kind: Kind::Rook }); // covers d9 and d8
        s.board[idx(1, 5)] = Some(Piece { side: 0, kind: Kind::Rook }); // covers f9 and f8
        // Red rook on e8 directly attacks the black general (check).
        // To make this a stalemate (no legal moves) without it being
        // capture-the-king on the previous move, we instead build:
        // black general at a9, two red rooks covering a8 and b9.
        s.board = [None; CELLS];
        s.board[idx(9, 0)] = Some(Piece { side: 1, kind: Kind::General });
        s.board[idx(0, 4)] = Some(Piece { side: 0, kind: Kind::General });
        // Red rook on b9 covers a9 (check) and b9-a9.
        // We want a position where after red's move, black has no moves.
        // Easier: red rook on h9 controls rank 9 (a9-h9). Black general
        // at a9 cannot go to b9 (rook controls), cannot go to a8 (palace
        // boundary: black palace cols 3-5 only). a9 isn't even in the
        // palace! Generals must stay in the palace, so we need general
        // inside the palace.
        //
        // Restart with general at f9 (col 5, row 9). Palace = cols 3-5,
        // rows 7-9. Adjacent palace squares: e9 (5-1=4), f8 (row 8 col 5).
        // Cover e9 with a rook on e1 (file e clear), cover f8 with rook
        // on f1 (file f clear). Black has no other pieces. Plus red
        // general at d0, plus a red pawn to make a tempo move.
        s.board = [None; CELLS];
        s.board[idx(9, 5)] = Some(Piece { side: 1, kind: Kind::General });
        s.board[idx(0, 3)] = Some(Piece { side: 0, kind: Kind::General });
        s.board[idx(1, 4)] = Some(Piece { side: 0, kind: Kind::Rook }); // covers e9
        s.board[idx(1, 5)] = Some(Piece { side: 0, kind: Kind::Rook }); // covers f8 and attacks f9
        // Red pawn at a4 to make a tempo move (a4 -> a5 across river).
        s.board[idx(4, 0)] = Some(Piece { side: 0, kind: Kind::Pawn });
        s.turn = 0;

        // Confirm black currently has no legal moves: the only piece is
        // the general at f9 with neighbours e9, f8 (both attacked) and
        // d9 (out of palace col-wise -> col 3 ok for palace, but distance
        // is 2 so not a 1-step move from f9). General's legal targets:
        // e9 (col 4, row 9, palace) and f8 (col 5, row 8, palace).
        // Both attacked by rooks -> in_check on the resulting position
        // (general moving into rook line of fire).
        assert!(legal_moves(&s.board, 1).is_empty(), "black should have no legal moves");

        // Red plays a tempo pawn push. After this red MOVE event the
        // server should emit WINNER 0 because black has no moves.
        drive(&mut s, "a", "MOVE", "a4 a5");
        assert_eq!(s.phase, Phase::Finished, "phase should be finished");
        assert_eq!(s.winner, Some(0));
    }

    // -------- replay reconstruction --------

    #[test]
    fn replay_reconstructs() {
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", ""),
            // Red pawn e3 -> e4
            ("MOVE", "a e3 e4"),
            // Black pawn e6 -> e5
            ("MOVE", "b e6 e5"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            apply(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Playing);
        // Red pawn now on e4.
        assert_eq!(
            s.board[idx(4, 4)],
            Some(Piece { side: 0, kind: Kind::Pawn })
        );
        // Black pawn now on e5.
        assert_eq!(
            s.board[idx(5, 4)],
            Some(Piece { side: 1, kind: Kind::Pawn })
        );
        // After two half-moves, red to move again.
        assert_eq!(s.turn, 0);
    }

    // -------- turn / off-turn rejection --------

    #[test]
    fn rejects_off_turn_move() {
        let s = started_game();
        // Black trying to move first.
        assert!(Xiangqi::validate(&s, "b", "MOVE", "e6 e5").is_err());
    }

    #[test]
    fn rejects_moving_opponents_piece() {
        let s = started_game();
        // Red trying to move a black piece.
        assert!(Xiangqi::validate(&s, "a", "MOVE", "e6 e5").is_err());
    }
}
