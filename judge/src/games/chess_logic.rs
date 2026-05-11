// Chess as RoomLogic.
//
// 8x8 board. Player 0 = white, player 1 = black (assigned by join order).
// Single source of truth: event log. The fold reapplies the same state
// machine the validator built the events from, so replay reconstructs
// castling rights, en passant target, halfmove clock, and the position
// repetition history exactly.

use crate::services::room::logic::RoomLogic;
use crate::services::storage::RoomSnapshot;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use std::collections::{BTreeMap, HashMap};

const MAX_PLAYERS: usize = 2;
const BOARD: usize = 64;
const FIFTY_MOVE_HALFMOVES: u32 = 100; // 50 full moves = 100 halfmoves

/// Per-game configuration parsed from the START / GAME_STARTED payload.
/// Format: `<pool_minutes> <per_turn_seconds> <blind 0|1>`. Missing or
/// invalid tokens fall back to defaults (no time limit, no blind).
///
/// Time enforcement happens in an external watcher (see services::time_pool);
/// the logic itself only stores the parameters so replay sees the same setup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeConfig {
    pub pool_minutes: u32,
    pub per_turn_seconds: u32,
    pub blind: bool,
}

impl TimeConfig {
    pub fn parse(payload: &str) -> Self {
        let mut it = payload.split_whitespace();
        let pool = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let per_turn = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let blind = matches!(it.next(), Some("1") | Some("true"));
        Self { pool_minutes: pool, per_turn_seconds: per_turn, blind }
    }

    pub fn render(self) -> String {
        format!(
            "{} {} {}",
            self.pool_minutes,
            self.per_turn_seconds,
            if self.blind { 1 } else { 0 }
        )
    }

    /// Like `render`, but appends a fresh blind-shuffle seed when blind
    /// mode is on. The seed is folded into the GAME_STARTED payload so
    /// every replay reproduces the same shuffled back rank.
    pub fn render_with_seed(self, seed: u64) -> String {
        if self.blind {
            format!(
                "{} {} 1 {}",
                self.pool_minutes, self.per_turn_seconds, seed
            )
        } else {
            self.render()
        }
    }
}

/// Pull the trailing seed token out of a GAME_STARTED payload that
/// was emitted in blind mode. Returns `None` when the seed is missing
/// or unparseable; the fold then falls back to a deterministic default
/// shuffle (seed 0).
pub fn parse_blind_seed(payload: &str) -> Option<u64> {
    payload.split_whitespace().nth(3).and_then(|s| s.parse::<u64>().ok())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    fn idx(self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
    fn from_idx(i: u8) -> Color {
        if i == 0 { Color::White } else { Color::Black }
    }
    fn other(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub type Piece = (Color, PieceType);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CastleRights {
    pub wk: bool,
    pub wq: bool,
    pub bk: bool,
    pub bq: bool,
}

#[derive(Debug, Clone)]
pub struct State {
    pub phase: Phase,
    pub players: Vec<String>,
    pub host: Option<String>,
    pub board: [Option<Piece>; BOARD],
    pub turn: Color,
    pub castle: CastleRights,
    pub en_passant: Option<u8>, // target square (the square the capturing pawn would land on)
    pub halfmove: u32,          // for 50-move rule
    pub winner: Option<u8>,     // None on draw or in-progress
    pub history: Vec<u64>,      // position hashes for threefold repetition
    /// Time + variant config supplied via START / GAME_STARTED. Stored on
    /// the state so clients can render the clock and the time-pool watcher
    /// can read its parameters from a freshly-loaded room.
    pub time_config: TimeConfig,
    /// Blind variant: maps a back-rank square to the piece type that
    /// piece pretends to be (its starting-square type) until it moves
    /// for the first time. The `state.board` itself holds the *true*
    /// piece type. Empty in non-blind games. Movement legality + attack
    /// detection use the facade overlay (see `display_board`); piece
    /// storage uses the true board.
    pub facade: BTreeMap<u8, PieceType>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            board: starting_board(),
            turn: Color::White,
            castle: CastleRights { wk: true, wq: true, bk: true, bq: true },
            en_passant: None,
            halfmove: 0,
            winner: None,
            history: Vec::new(),
            time_config: TimeConfig::default(),
            facade: BTreeMap::new(),
        }
    }
}

impl State {
    fn player_index(&self, player_id: &str) -> Option<u8> {
        self.players.iter().position(|p| p == player_id).map(|i| i as u8)
    }

    /// Clone with the board field swapped out. Used to feed the legal-
    /// move generator a facade-overlaid view without mutating self.
    fn with_board(&self, board: [Option<Piece>; BOARD]) -> State {
        let mut s = self.clone();
        s.board = board;
        s
    }
    fn reset_for_new_game(&mut self) {
        self.board = starting_board();
        self.turn = Color::White;
        self.castle = CastleRights { wk: true, wq: true, bk: true, bq: true };
        self.en_passant = None;
        self.halfmove = 0;
        self.winner = None;
        self.facade = BTreeMap::new();
        self.history = vec![position_hash(&self.board, self.turn, self.castle, self.en_passant)];
    }
}

/// Back-rank squares whose true type is shuffled when blind mode is on.
/// Pawns + king stay as themselves (king is the variant's only fixed
/// landmark; pawns avoid the promotion/en-passant edge cases that come
/// with mid-life facade swaps).
const BLIND_SQUARES_WHITE: [u8; 7] = [0, 1, 2, 3, 5, 6, 7]; // a1..h1 minus e1
const BLIND_SQUARES_BLACK: [u8; 7] = [56, 57, 58, 59, 61, 62, 63]; // a8..h8 minus e8

/// True piece types these squares hold in the standard starting layout.
/// Same order as BLIND_SQUARES_*; reused as the pool that gets shuffled
/// in blind mode.
const BLIND_TYPES: [PieceType; 7] = [
    PieceType::Rook,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Queen,
    PieceType::Bishop,
    PieceType::Knight,
    PieceType::Rook,
];

/// Apply a blind shuffle to the back rank seeded by `seed`. Sets the
/// true types on `state.board` and populates `state.facade` so each
/// shuffled square pretends to be its starting-position piece until it
/// moves. Pawns and kings are left untouched.
fn apply_blind_shuffle(state: &mut State, seed: u64) {
    let mut rng = StdRng::seed_from_u64(seed);
    state.facade.clear();
    for (color, squares) in [
        (Color::White, &BLIND_SQUARES_WHITE),
        (Color::Black, &BLIND_SQUARES_BLACK),
    ] {
        let mut pool = BLIND_TYPES.to_vec();
        pool.shuffle(&mut rng);
        for (i, &square) in squares.iter().enumerate() {
            state.board[square as usize] = Some((color, pool[i]));
            state.facade.insert(square, BLIND_TYPES[i]);
        }
    }
    // Reset the position-hash baseline so threefold detection works
    // against the freshly-shuffled true board.
    state.history = vec![position_hash(&state.board, state.turn, state.castle, state.en_passant)];
}

/// Project the true board through the facade overlay: every square
/// whose facade is set shows the facade type instead of its true type.
/// Used wherever movement legality or attack detection runs — both
/// sides see the facade until pieces reveal themselves.
pub fn display_board(state: &State) -> [Option<Piece>; BOARD] {
    let mut b = state.board;
    for (sq_idx, facade_type) in &state.facade {
        if let Some((color, _)) = b[*sq_idx as usize] {
            b[*sq_idx as usize] = Some((color, *facade_type));
        }
    }
    b
}


pub struct Chess;

// ---------- board layout ----------
//
// We index 0..63 with 0 = a1, 7 = h1, 56 = a8, 63 = h8. file = sq % 8,
// rank = sq / 8. Going from white's side: file a..h is left..right;
// rank 1..8 is bottom..top. White pieces start on ranks 0..1, black on
// ranks 6..7. This keeps algebraic conversion trivial.

fn sq(file: u8, rank: u8) -> u8 {
    rank * 8 + file
}

fn file_of(s: u8) -> u8 {
    s % 8
}

fn rank_of(s: u8) -> u8 {
    s / 8
}

pub fn parse_square(s: &str) -> Option<u8> {
    let b = s.as_bytes();
    if b.len() != 2 {
        return None;
    }
    let f = b[0];
    let r = b[1];
    if !(b'a'..=b'h').contains(&f) || !(b'1'..=b'8').contains(&r) {
        return None;
    }
    Some(sq(f - b'a', r - b'1'))
}

pub fn square_str(s: u8) -> String {
    let f = (b'a' + file_of(s)) as char;
    let r = (b'1' + rank_of(s)) as char;
    format!("{f}{r}")
}

fn starting_board() -> [Option<Piece>; BOARD] {
    let mut b: [Option<Piece>; BOARD] = [None; BOARD];
    use Color::*;
    use PieceType::*;
    let back = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
    for f in 0..8u8 {
        b[sq(f, 0) as usize] = Some((White, back[f as usize]));
        b[sq(f, 1) as usize] = Some((White, Pawn));
        b[sq(f, 6) as usize] = Some((Black, Pawn));
        b[sq(f, 7) as usize] = Some((Black, back[f as usize]));
    }
    b
}

// ---------- attack / movement ----------

const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (1, 2), (2, 1), (2, -1), (1, -2),
    (-1, -2), (-2, -1), (-2, 1), (-1, 2),
];
const KING_DELTAS: [(i8, i8); 8] = [
    (1, 0), (1, 1), (0, 1), (-1, 1),
    (-1, 0), (-1, -1), (0, -1), (1, -1),
];
const BISHOP_DIRS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

fn add(s: u8, df: i8, dr: i8) -> Option<u8> {
    let f = file_of(s) as i8 + df;
    let r = rank_of(s) as i8 + dr;
    if (0..8).contains(&f) && (0..8).contains(&r) {
        Some(sq(f as u8, r as u8))
    } else {
        None
    }
}

/// Squares attacked by `color`'s pieces. Pawn attacks are diagonal
/// only — pawn pushes do not attack.
fn is_attacked(board: &[Option<Piece>; BOARD], target: u8, by: Color) -> bool {
    use PieceType::*;
    // pawns
    let pawn_dr: i8 = if by == Color::White { 1 } else { -1 };
    for df in [-1i8, 1] {
        if let Some(s) = add(target, -df, -pawn_dr) {
            if board[s as usize] == Some((by, Pawn)) {
                return true;
            }
        }
    }
    // knights
    for (df, dr) in KNIGHT_DELTAS {
        if let Some(s) = add(target, df, dr) {
            if board[s as usize] == Some((by, Knight)) {
                return true;
            }
        }
    }
    // king (adjacent)
    for (df, dr) in KING_DELTAS {
        if let Some(s) = add(target, df, dr) {
            if board[s as usize] == Some((by, King)) {
                return true;
            }
        }
    }
    // bishops + queens (diagonal rays)
    for (df, dr) in BISHOP_DIRS {
        let mut s = target;
        loop {
            let Some(n) = add(s, df, dr) else { break };
            s = n;
            if let Some((c, pt)) = board[s as usize] {
                if c == by && (pt == Bishop || pt == Queen) {
                    return true;
                }
                break;
            }
        }
    }
    // rooks + queens (orthogonal rays)
    for (df, dr) in ROOK_DIRS {
        let mut s = target;
        loop {
            let Some(n) = add(s, df, dr) else { break };
            s = n;
            if let Some((c, pt)) = board[s as usize] {
                if c == by && (pt == Rook || pt == Queen) {
                    return true;
                }
                break;
            }
        }
    }
    false
}

fn king_square(board: &[Option<Piece>; BOARD], color: Color) -> Option<u8> {
    for (i, p) in board.iter().enumerate() {
        if *p == Some((color, PieceType::King)) {
            return Some(i as u8);
        }
    }
    None
}

fn in_check(board: &[Option<Piece>; BOARD], color: Color) -> bool {
    match king_square(board, color) {
        Some(k) => is_attacked(board, k, color.other()),
        None => false,
    }
}

// ---------- move application ----------

/// Tag describing how a move modifies the board beyond `board[from] →
/// board[to]`. Used both as the protocol flag and as the apply rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Normal,
    CastleK,
    CastleQ,
    EnPassant,
    Promote(PieceType),
}

impl MoveFlag {
    fn as_str(self) -> String {
        match self {
            MoveFlag::Normal => "-".into(),
            MoveFlag::CastleK => "K".into(),
            MoveFlag::CastleQ => "Q".into(),
            MoveFlag::EnPassant => "E".into(),
            MoveFlag::Promote(pt) => promo_char(pt).to_string(),
        }
    }
    fn parse(s: &str) -> Option<MoveFlag> {
        match s {
            "-" => Some(MoveFlag::Normal),
            "K" => Some(MoveFlag::CastleK),
            "Q" => Some(MoveFlag::CastleQ),
            "E" => Some(MoveFlag::EnPassant),
            "q" => Some(MoveFlag::Promote(PieceType::Queen)),
            "r" => Some(MoveFlag::Promote(PieceType::Rook)),
            "b" => Some(MoveFlag::Promote(PieceType::Bishop)),
            "n" => Some(MoveFlag::Promote(PieceType::Knight)),
            _ => None,
        }
    }
}

fn promo_char(pt: PieceType) -> char {
    match pt {
        PieceType::Queen => 'q',
        PieceType::Rook => 'r',
        PieceType::Bishop => 'b',
        PieceType::Knight => 'n',
        _ => '?',
    }
}

fn parse_promo(s: &str) -> Option<PieceType> {
    match s {
        "q" | "Q" => Some(PieceType::Queen),
        "r" | "R" => Some(PieceType::Rook),
        "b" | "B" => Some(PieceType::Bishop),
        "n" | "N" => Some(PieceType::Knight),
        _ => None,
    }
}

/// Apply `(from, to, flag)` to a fresh board copy and return the new
/// board, captured-piece flag (for halfmove reset), and pawn-move flag.
fn apply_move(
    board: &[Option<Piece>; BOARD],
    from: u8,
    to: u8,
    flag: MoveFlag,
) -> ([Option<Piece>; BOARD], bool, bool) {
    let mut b = *board;
    let piece = b[from as usize];
    let capture = b[to as usize].is_some() || flag == MoveFlag::EnPassant;
    let pawn_move = matches!(piece, Some((_, PieceType::Pawn)));

    b[from as usize] = None;
    b[to as usize] = piece;

    match flag {
        MoveFlag::CastleK => {
            // King moved e→g; rook moves h→f on the same rank.
            let rank = rank_of(to);
            let rook = b[sq(7, rank) as usize];
            b[sq(7, rank) as usize] = None;
            b[sq(5, rank) as usize] = rook;
        }
        MoveFlag::CastleQ => {
            // King moved e→c; rook moves a→d on the same rank.
            let rank = rank_of(to);
            let rook = b[sq(0, rank) as usize];
            b[sq(0, rank) as usize] = None;
            b[sq(3, rank) as usize] = rook;
        }
        MoveFlag::EnPassant => {
            // Captured pawn sits behind `to` from the mover's POV.
            if let Some((c, _)) = piece {
                let cap_rank = if c == Color::White { rank_of(to) - 1 } else { rank_of(to) + 1 };
                b[sq(file_of(to), cap_rank) as usize] = None;
            }
        }
        MoveFlag::Promote(pt) => {
            if let Some((c, _)) = piece {
                b[to as usize] = Some((c, pt));
            }
        }
        MoveFlag::Normal => {}
    }
    (b, capture, pawn_move)
}

/// Update castling rights after a move. Rights are lost if the king
/// moves, the rook moves from its home square, or the rook is captured
/// on its home square.
fn update_castle_rights(rights: CastleRights, from: u8, to: u8, piece: Piece) -> CastleRights {
    let mut r = rights;
    match piece {
        (Color::White, PieceType::King) => { r.wk = false; r.wq = false; }
        (Color::Black, PieceType::King) => { r.bk = false; r.bq = false; }
        _ => {}
    }
    // Rook moved off home square OR rook captured on home square.
    for s in [from, to] {
        if s == sq(0, 0) { r.wq = false; }
        if s == sq(7, 0) { r.wk = false; }
        if s == sq(0, 7) { r.bq = false; }
        if s == sq(7, 7) { r.bk = false; }
    }
    r
}

/// New en-passant target after a pawn double step; otherwise `None`.
fn ep_after_move(piece: Piece, from: u8, to: u8) -> Option<u8> {
    if piece.1 != PieceType::Pawn {
        return None;
    }
    let dr = rank_of(to) as i8 - rank_of(from) as i8;
    if dr.abs() != 2 {
        return None;
    }
    // The target sits between from and to.
    Some(sq(file_of(from), (rank_of(from) as i8 + dr / 2) as u8))
}

// ---------- legal move generation ----------

/// Generate all legal pseudo-encoded moves for `color` in `state`.
/// Includes castling, en passant, promotions (one entry per promotion
/// piece), and rejects moves that leave the king in check.
fn legal_moves(state: &State, color: Color) -> Vec<(u8, u8, MoveFlag)> {
    let mut out: Vec<(u8, u8, MoveFlag)> = Vec::new();
    for from in 0..BOARD as u8 {
        let Some((c, pt)) = state.board[from as usize] else { continue };
        if c != color {
            continue;
        }
        gen_piece_moves(state, from, c, pt, &mut out);
    }
    // Filter out moves that leave own king in check.
    out.retain(|&(f, t, fl)| {
        let (b2, _, _) = apply_move(&state.board, f, t, fl);
        !in_check(&b2, color)
    });
    out
}

fn gen_piece_moves(
    state: &State,
    from: u8,
    c: Color,
    pt: PieceType,
    out: &mut Vec<(u8, u8, MoveFlag)>,
) {
    use PieceType::*;
    match pt {
        Pawn => gen_pawn(state, from, c, out),
        Knight => {
            for (df, dr) in KNIGHT_DELTAS {
                if let Some(t) = add(from, df, dr) {
                    if state.board[t as usize].map(|(oc, _)| oc) != Some(c) {
                        out.push((from, t, MoveFlag::Normal));
                    }
                }
            }
        }
        Bishop => slide(state, from, c, &BISHOP_DIRS, out),
        Rook => slide(state, from, c, &ROOK_DIRS, out),
        Queen => {
            slide(state, from, c, &BISHOP_DIRS, out);
            slide(state, from, c, &ROOK_DIRS, out);
        }
        King => {
            for (df, dr) in KING_DELTAS {
                if let Some(t) = add(from, df, dr) {
                    if state.board[t as usize].map(|(oc, _)| oc) != Some(c) {
                        out.push((from, t, MoveFlag::Normal));
                    }
                }
            }
            gen_castles(state, from, c, out);
        }
    }
}

fn slide(
    state: &State,
    from: u8,
    c: Color,
    dirs: &[(i8, i8)],
    out: &mut Vec<(u8, u8, MoveFlag)>,
) {
    for (df, dr) in dirs {
        let mut s = from;
        loop {
            let Some(n) = add(s, *df, *dr) else { break };
            s = n;
            match state.board[s as usize] {
                None => out.push((from, s, MoveFlag::Normal)),
                Some((oc, _)) => {
                    if oc != c {
                        out.push((from, s, MoveFlag::Normal));
                    }
                    break;
                }
            }
        }
    }
}

fn gen_pawn(state: &State, from: u8, c: Color, out: &mut Vec<(u8, u8, MoveFlag)>) {
    let dir: i8 = if c == Color::White { 1 } else { -1 };
    let start_rank: u8 = if c == Color::White { 1 } else { 6 };
    let promo_rank: u8 = if c == Color::White { 7 } else { 0 };

    // Single push.
    if let Some(t) = add(from, 0, dir) {
        if state.board[t as usize].is_none() {
            push_pawn(from, t, promo_rank, out);
            // Double push only from start rank, both squares empty.
            if rank_of(from) == start_rank {
                if let Some(t2) = add(from, 0, dir * 2) {
                    if state.board[t2 as usize].is_none() {
                        out.push((from, t2, MoveFlag::Normal));
                    }
                }
            }
        }
    }
    // Captures.
    for df in [-1i8, 1] {
        if let Some(t) = add(from, df, dir) {
            if let Some((oc, _)) = state.board[t as usize] {
                if oc != c {
                    push_pawn(from, t, promo_rank, out);
                }
            }
            if state.en_passant == Some(t) {
                out.push((from, t, MoveFlag::EnPassant));
            }
        }
    }
}

fn push_pawn(from: u8, to: u8, promo_rank: u8, out: &mut Vec<(u8, u8, MoveFlag)>) {
    if rank_of(to) == promo_rank {
        for pt in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
            out.push((from, to, MoveFlag::Promote(pt)));
        }
    } else {
        out.push((from, to, MoveFlag::Normal));
    }
}

fn gen_castles(state: &State, from: u8, c: Color, out: &mut Vec<(u8, u8, MoveFlag)>) {
    let rank = if c == Color::White { 0 } else { 7 };
    if from != sq(4, rank) {
        return;
    }
    if in_check(&state.board, c) {
        return;
    }
    let (kside, qside) = match c {
        Color::White => (state.castle.wk, state.castle.wq),
        Color::Black => (state.castle.bk, state.castle.bq),
    };
    let opp = c.other();
    if kside
        && state.board[sq(5, rank) as usize].is_none()
        && state.board[sq(6, rank) as usize].is_none()
        && state.board[sq(7, rank) as usize] == Some((c, PieceType::Rook))
        && !is_attacked(&state.board, sq(5, rank), opp)
        && !is_attacked(&state.board, sq(6, rank), opp)
    {
        out.push((from, sq(6, rank), MoveFlag::CastleK));
    }
    if qside
        && state.board[sq(1, rank) as usize].is_none()
        && state.board[sq(2, rank) as usize].is_none()
        && state.board[sq(3, rank) as usize].is_none()
        && state.board[sq(0, rank) as usize] == Some((c, PieceType::Rook))
        && !is_attacked(&state.board, sq(3, rank), opp)
        && !is_attacked(&state.board, sq(2, rank), opp)
    {
        out.push((from, sq(2, rank), MoveFlag::CastleQ));
    }
}

// ---------- terminal detection ----------

fn insufficient_material(board: &[Option<Piece>; BOARD]) -> bool {
    use PieceType::*;
    let mut white_minors = 0u32;
    let mut black_minors = 0u32;
    for p in board.iter().flatten() {
        match p.1 {
            King => {}
            Knight | Bishop => {
                if p.0 == Color::White { white_minors += 1; } else { black_minors += 1; }
            }
            // Any pawn / rook / queen → mate is possible.
            _ => return false,
        }
    }
    // K vs K, K+minor vs K (both directions). Any larger material set
    // can in principle mate, so we only flag the trivial cases.
    matches!((white_minors, black_minors), (0, 0) | (1, 0) | (0, 1))
}

fn position_hash(
    board: &[Option<Piece>; BOARD],
    turn: Color,
    castle: CastleRights,
    ep: Option<u8>,
) -> u64 {
    // FNV-1a over the canonical position fields. Threefold uses this
    // as the equivalence key — it ignores history beyond the position
    // itself, which matches the FIDE rule.
    let mut h: u64 = 0xcbf29ce484222325;
    let mut mix = |x: u64| {
        let mut h2 = h;
        for b in x.to_le_bytes() {
            h2 ^= b as u64;
            h2 = h2.wrapping_mul(0x100000001b3);
        }
        h = h2;
    };
    for (i, p) in board.iter().enumerate() {
        let v: u64 = match p {
            None => 0,
            Some((Color::White, pt)) => 1 + piece_code(*pt) as u64,
            Some((Color::Black, pt)) => 7 + piece_code(*pt) as u64,
        };
        mix((i as u64) << 8 | v);
    }
    mix(turn.idx() as u64);
    mix((castle.wk as u64) | ((castle.wq as u64) << 1)
        | ((castle.bk as u64) << 2) | ((castle.bq as u64) << 3));
    mix(ep.map(|e| e as u64 + 1).unwrap_or(0));
    h
}

fn piece_code(pt: PieceType) -> u8 {
    use PieceType::*;
    match pt {
        Pawn => 0, Knight => 1, Bishop => 2, Rook => 3, Queen => 4, King => 5,
    }
}

fn threefold(history: &[u64], current: u64) -> bool {
    let mut count = 0;
    for &h in history {
        if h == current {
            count += 1;
            if count >= 3 {
                return true;
            }
        }
    }
    false
}

// ---------- RoomLogic impl ----------

impl RoomLogic for Chess {
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
                state.reset_for_new_game();
                state.time_config = TimeConfig::parse(payload);
                if state.time_config.blind {
                    let seed = parse_blind_seed(payload).unwrap_or(0);
                    apply_blind_shuffle(state, seed);
                }
            }
            "MOVE" => {
                // payload: "<pid> <from> <to> <flag>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() != 4 {
                    return;
                }
                let Some(from) = parse_square(parts[1]) else { return };
                let Some(to) = parse_square(parts[2]) else { return };
                let Some(flag) = MoveFlag::parse(parts[3]) else { return };
                let Some(true_piece) = state.board[from as usize] else { return };

                // Castling-rights + en-passant bookkeeping uses the
                // displayed (facade) piece type so blind games behave
                // the same on replay as on validate.
                let display_pt = state
                    .facade
                    .get(&from)
                    .copied()
                    .unwrap_or(true_piece.1);
                let display_piece = (true_piece.0, display_pt);

                let (b2, capture, pawn_move) = apply_move(&state.board, from, to, flag);
                state.board = b2;
                state.castle = update_castle_rights(state.castle, from, to, display_piece);
                state.en_passant = ep_after_move(display_piece, from, to);
                if capture || pawn_move {
                    state.halfmove = 0;
                    // Repetition counter resets on irreversible moves.
                    state.history.clear();
                } else {
                    state.halfmove += 1;
                }
                let mover = true_piece.0;
                let _revealed = update_facade_post_move(&mut state.facade, mover, from, to);
                state.turn = state.turn.other();
                let display_b = overlay_facade(&state.board, &state.facade);
                let h = position_hash(&display_b, state.turn, state.castle, state.en_passant);
                state.history.push(h);
            }
            "PIECE_REVEALED" => {} // bookkeeping; fold("MOVE") already updated facade.
            "CHECK" => {} // pure notification; state already in check.
            "WINNER" => {
                if let Ok(w) = payload.trim().parse::<u8>() {
                    state.winner = Some(w);
                    state.phase = Phase::Finished;
                }
            }
            "DRAW" => {
                state.winner = None;
                state.phase = Phase::Finished;
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
                    if let Some(h) =
                        state.players.iter().find(|p| p.as_str() != player).cloned()
                    {
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
                let cfg = TimeConfig::parse(payload);
                let payload = if cfg.blind {
                    cfg.render_with_seed(rand::rng().random::<u64>())
                } else {
                    cfg.render()
                };
                Ok(vec![("GAME_STARTED".into(), payload)])
            }
            "MOVE" => validate_move(state, player, payload),
            _ => Err(format!("unknown action: {kind}")),
        }
    }

    fn max_players() -> usize {
        MAX_PLAYERS
    }

    fn game_id() -> &'static str {
        "chess"
    }

    fn pending_players(state: &Self::State) -> Vec<String> {
        if state.phase != Phase::Playing {
            return Vec::new();
        }
        state
            .players
            .get(state.turn.idx() as usize)
            .cloned()
            .into_iter()
            .collect()
    }

    fn time_pool_seconds(state: &Self::State) -> Option<u64> {
        let m = state.time_config.pool_minutes;
        if m == 0 { None } else { Some(m as u64 * 60) }
    }

    fn per_turn_seconds(state: &Self::State) -> Option<u64> {
        let s = state.time_config.per_turn_seconds;
        if s == 0 { None } else { Some(s as u64) }
    }

    fn snapshot(state: &Self::State) -> RoomSnapshot {
        RoomSnapshot {
            phase: phase_str(&state.phase),
            host: state.host.clone(),
            players: state.players.clone(),
        }
    }
}

fn validate_move(
    state: &State,
    player: &str,
    payload: &str,
) -> Result<Vec<(String, String)>, String> {
    if state.phase != Phase::Playing {
        return Err("not playing".into());
    }
    let idx = state.player_index(player).ok_or_else(|| "not in room".to_string())?;
    let mover = Color::from_idx(idx);
    if state.turn != mover {
        return Err("not your turn".into());
    }

    let parts: Vec<&str> = payload.split_whitespace().collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err("expected: MOVE <from> <to> [promo]".into());
    }
    let from = parse_square(parts[0]).ok_or_else(|| "bad from square".to_string())?;
    let to = parse_square(parts[1]).ok_or_else(|| "bad to square".to_string())?;
    let promo = parts.get(2).and_then(|s| parse_promo(s));

    // In blind mode the piece on `from` may have a hidden true type;
    // legality + castling-rights bookkeeping always run against the
    // displayed (facade) piece — that's what both players see and how
    // the chess rules apply until the piece is revealed.
    let display = display_board(state);
    let display_piece = display[from as usize]
        .ok_or_else(|| "no piece on from square".to_string())?;
    if display_piece.0 != mover {
        return Err("not your piece".into());
    }

    // Find the matching legal move. We trust the legal-move generator
    // as the single source of truth for legality so castling, en
    // passant, and check-evasion all share one path.
    let display_state = state.with_board(display);
    let legals = legal_moves(&display_state, mover);
    let mut chosen: Option<(u8, u8, MoveFlag)> = None;
    for (f, t, fl) in &legals {
        if *f != from || *t != to {
            continue;
        }
        match fl {
            MoveFlag::Promote(pt) => {
                let want = promo.ok_or_else(|| "promotion piece required".to_string())?;
                if *pt == want {
                    chosen = Some((*f, *t, *fl));
                    break;
                }
            }
            _ => {
                if promo.is_some() {
                    // Promotion specified for non-promotion move: ignore silently per spec.
                }
                chosen = Some((*f, *t, *fl));
                break;
            }
        }
    }
    let (f, t, fl) = chosen.ok_or_else(|| "illegal move".to_string())?;

    // Apply provisionally to detect terminal conditions. We mutate the
    // *true* board so the destination square holds the real piece type,
    // not the facade — that matters once the piece reveals.
    let (b2, capture, pawn_move) = apply_move(&state.board, f, t, fl);
    let new_castle = update_castle_rights(state.castle, f, t, display_piece);
    let new_ep = ep_after_move(display_piece, f, t);
    let new_halfmove = if capture || pawn_move { 0 } else { state.halfmove + 1 };
    let new_turn = mover.other();

    // Compute the post-move facade so the opponent's legality check sees
    // the same board the players will see after this move resolves.
    let mut next_facade = state.facade.clone();
    let revealed_squares = update_facade_post_move(&mut next_facade, mover, f, t);

    let display_b2 = overlay_facade(&b2, &next_facade);
    let new_hash = position_hash(&display_b2, new_turn, new_castle, new_ep);
    let opponent_in_check = in_check(&display_b2, new_turn);
    let next_state = synthetic_state(
        &b2, new_turn, new_castle, new_ep, &state.history,
        new_halfmove, capture || pawn_move, new_hash, next_facade,
    );
    let opp_legals = legal_moves(&next_state, new_turn);
    let opponent_has_move = !opp_legals.is_empty();

    let mut out = vec![(
        "MOVE".into(),
        format!("{player} {} {} {}", square_str(f), square_str(t), fl.as_str()),
    )];

    // Emit one PIECE_REVEALED per square that just became visible. The
    // payload carries the *true* type so clients can update their board
    // mirror without having to know the seeded shuffle.
    for sq_idx in revealed_squares {
        if let Some((_, true_type)) = b2[sq_idx as usize] {
            out.push((
                "PIECE_REVEALED".into(),
                format!("{} {}", square_str(sq_idx), piecetype_letter(true_type)),
            ));
        }
    }

    if opponent_in_check && !opponent_has_move {
        out.push(("WINNER".into(), mover.idx().to_string()));
    } else if !opponent_in_check && !opponent_has_move {
        out.push(("DRAW".into(), "STALEMATE".into()));
    } else if new_halfmove >= FIFTY_MOVE_HALFMOVES {
        out.push(("DRAW".into(), "FIFTY_MOVE".into()));
    } else if threefold(&next_state.history, new_hash) {
        out.push(("DRAW".into(), "THREEFOLD".into()));
    } else if insufficient_material(&display_b2) {
        out.push(("DRAW".into(), "INSUFFICIENT".into()));
    } else if opponent_in_check {
        // Plain check (no mate, no other terminal). Notify after MOVE.
        out.push(("CHECK".into(), new_turn.idx().to_string()));
    }
    Ok(out)
}

/// Strip facade entries for `from` and `to`, then auto-reveal the
/// mover's last hidden piece if exactly one remains. Returns squares
/// that newly became revealed, in emission order, so the validator can
/// produce matching `PIECE_REVEALED` events. Pure: works on any facade
/// map, no `&mut state` required.
fn update_facade_post_move(
    facade: &mut BTreeMap<u8, PieceType>,
    mover: Color,
    from: u8,
    to: u8,
) -> Vec<u8> {
    let mut revealed = Vec::new();
    if facade.remove(&from).is_some() {
        revealed.push(to); // piece now lives on `to`
    }
    if facade.remove(&to).is_some() && !revealed.contains(&to) {
        revealed.push(to);
    }
    let mover_squares: &[u8] = match mover {
        Color::White => &BLIND_SQUARES_WHITE,
        Color::Black => &BLIND_SQUARES_BLACK,
    };
    let remaining: Vec<u8> = mover_squares
        .iter()
        .copied()
        .filter(|sq_idx| facade.contains_key(sq_idx))
        .collect();
    if remaining.len() == 1 {
        let last = remaining[0];
        facade.remove(&last);
        if !revealed.contains(&last) {
            revealed.push(last);
        }
    }
    revealed
}

fn overlay_facade(
    board: &[Option<Piece>; BOARD],
    facade: &BTreeMap<u8, PieceType>,
) -> [Option<Piece>; BOARD] {
    let mut b = *board;
    for (sq_idx, facade_type) in facade {
        if let Some((color, _)) = b[*sq_idx as usize] {
            b[*sq_idx as usize] = Some((color, *facade_type));
        }
    }
    b
}

fn piecetype_letter(pt: PieceType) -> &'static str {
    match pt {
        PieceType::Pawn => "P",
        PieceType::Knight => "N",
        PieceType::Bishop => "B",
        PieceType::Rook => "R",
        PieceType::Queen => "Q",
        PieceType::King => "K",
    }
}


/// Build a transient State just for the legal-move check after a move.
/// Only the fields legal_moves consults need to be accurate.
#[allow(clippy::too_many_arguments)]
fn synthetic_state(
    board: &[Option<Piece>; BOARD],
    turn: Color,
    castle: CastleRights,
    ep: Option<u8>,
    prev_history: &[u64],
    halfmove: u32,
    irreversible: bool,
    new_hash: u64,
    facade: BTreeMap<u8, PieceType>,
) -> State {
    let mut history = if irreversible { Vec::new() } else { prev_history.to_vec() };
    history.push(new_hash);
    State {
        phase: Phase::Playing,
        players: Vec::new(),
        host: None,
        board: *board,
        turn,
        castle,
        en_passant: ep,
        halfmove,
        winner: None,
        history,
        time_config: TimeConfig::default(),
        facade,
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

// ---------- legal-move helper for downstream callers ----------

/// All `(from, to, flag)` triples that are currently legal for the side
/// to move. Returned in board-iteration order. Exposed so the
/// playground bot and (eventually) any in-process consumer can reuse
/// the same generator the validator relies on.
pub fn legal_moves_for_turn(state: &State) -> Vec<(u8, u8, MoveFlag)> {
    legal_moves(state, state.turn)
}

/// Map of square → piece for inspection. Useful for tests and bots.
pub fn board_map(state: &State) -> HashMap<u8, Piece> {
    let mut m = HashMap::new();
    for (i, p) in state.board.iter().enumerate() {
        if let Some(p) = p {
            m.insert(i as u8, *p);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drive(s: &mut State, player: &str, kind: &str, payload: &str) {
        let evs = Chess::validate(s, player, kind, payload).expect("validate ok");
        for (k, p) in &evs {
            Chess::fold(s, k, p);
        }
    }

    fn join_and_start() -> State {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        s
    }

    #[test]
    fn scholars_mate() {
        // 1. e4 e5  2. Bc4 Nc6  3. Qh5 Nf6??  4. Qxf7#
        let mut s = join_and_start();
        drive(&mut s, "a", "MOVE", "e2 e4");
        drive(&mut s, "b", "MOVE", "e7 e5");
        drive(&mut s, "a", "MOVE", "f1 c4");
        drive(&mut s, "b", "MOVE", "b8 c6");
        drive(&mut s, "a", "MOVE", "d1 h5");
        drive(&mut s, "b", "MOVE", "g8 f6");
        drive(&mut s, "a", "MOVE", "h5 f7");
        assert_eq!(s.phase, Phase::Finished, "should be finished after mate");
        assert_eq!(s.winner, Some(0));
    }

    #[test]
    fn kingside_castle() {
        let mut s = join_and_start();
        // Open up f1 bishop and g1 knight for white.
        drive(&mut s, "a", "MOVE", "e2 e4");
        drive(&mut s, "b", "MOVE", "e7 e5");
        drive(&mut s, "a", "MOVE", "g1 f3");
        drive(&mut s, "b", "MOVE", "b8 c6");
        drive(&mut s, "a", "MOVE", "f1 c4");
        drive(&mut s, "b", "MOVE", "g8 f6");
        // White castles kingside.
        let evs = Chess::validate(&s, "a", "MOVE", "e1 g1").expect("castle legal");
        let move_ev = evs.iter().find(|(k, _)| k == "MOVE").expect("emits MOVE");
        assert!(move_ev.1.ends_with(" K"), "kingside castle flag K");
        for (k, p) in &evs {
            Chess::fold(&mut s, k, p);
        }
        assert_eq!(s.board[sq(6, 0) as usize], Some((Color::White, PieceType::King)));
        assert_eq!(s.board[sq(5, 0) as usize], Some((Color::White, PieceType::Rook)));
        assert!(!s.castle.wk && !s.castle.wq);
    }

    #[test]
    fn en_passant_capture() {
        let mut s = join_and_start();
        // 1. e4 a6 (waiting)  2. e5 d5 (black double-step adjacent)  3. exd6 e.p.
        drive(&mut s, "a", "MOVE", "e2 e4");
        drive(&mut s, "b", "MOVE", "a7 a6");
        drive(&mut s, "a", "MOVE", "e4 e5");
        drive(&mut s, "b", "MOVE", "d7 d5");
        // En passant target should be d6.
        assert_eq!(s.en_passant, Some(parse_square("d6").unwrap()));
        let evs = Chess::validate(&s, "a", "MOVE", "e5 d6").expect("ep legal");
        let move_ev = evs.iter().find(|(k, _)| k == "MOVE").expect("emits MOVE");
        assert!(move_ev.1.ends_with(" E"), "en passant flag E");
        for (k, p) in &evs {
            Chess::fold(&mut s, k, p);
        }
        assert_eq!(s.board[sq(3, 5) as usize], Some((Color::White, PieceType::Pawn))); // d6 occupied
        assert_eq!(s.board[sq(3, 4) as usize], None); // d5 emptied (captured pawn)
    }

    #[test]
    fn rejects_move_into_check() {
        // White king on e1 cannot step to f1 if a black rook controls f1.
        let mut s = State::default();
        s.phase = Phase::Playing;
        s.board = [None; BOARD];
        s.board[sq(4, 0) as usize] = Some((Color::White, PieceType::King));
        s.board[sq(4, 7) as usize] = Some((Color::Black, PieceType::King));
        s.board[sq(5, 7) as usize] = Some((Color::Black, PieceType::Rook)); // f8 covers f-file
        s.players = vec!["a".into(), "b".into()];
        s.host = Some("a".into());
        s.turn = Color::White;
        s.history = vec![position_hash(&s.board, s.turn, s.castle, s.en_passant)];

        let r = Chess::validate(&s, "a", "MOVE", "e1 f1");
        assert!(r.is_err(), "king cannot walk onto attacked square");
        // Sanity: stepping to d1 (off the f-file) is fine.
        let ok = Chess::validate(&s, "a", "MOVE", "e1 d1");
        assert!(ok.is_ok(), "king may step to safe square");
    }

    #[test]
    fn rejects_leaving_king_in_check() {
        // White Bb5 pins Bd7 to Ke8 along the a4-e8 diagonal. Moving the
        // pinned bishop off the diagonal must be rejected.
        let mut s = join_and_start();
        drive(&mut s, "a", "MOVE", "e2 e4");
        drive(&mut s, "b", "MOVE", "d7 d6");
        drive(&mut s, "a", "MOVE", "f1 b5");
        drive(&mut s, "b", "MOVE", "c8 d7");
        let r = Chess::validate(&s, "b", "MOVE", "d7 e6");
        assert!(r.is_err(), "moving pinned bishop must be rejected");
    }

    #[test]
    fn stalemate_position() {
        // Classic stalemate trigger: Kc2 + Qb4 vs Ka1, black to move plays
        // Qb3, leaving white with no legal move and not in check.
        let mut s = State::default();
        s.phase = Phase::Playing;
        s.board = [None; BOARD];
        s.board[sq(0, 0) as usize] = Some((Color::White, PieceType::King));
        s.board[sq(2, 1) as usize] = Some((Color::Black, PieceType::King));
        s.board[sq(1, 3) as usize] = Some((Color::Black, PieceType::Queen)); // b4
        s.players = vec!["a".into(), "b".into()];
        s.host = Some("a".into());
        s.turn = Color::Black;
        s.castle = CastleRights::default();
        s.history = vec![position_hash(&s.board, s.turn, s.castle, s.en_passant)];

        let evs = Chess::validate(&s, "b", "MOVE", "b4 b3").expect("Qb3 legal");
        let draw = evs.iter().find(|(k, _)| k == "DRAW").expect("emits DRAW");
        assert_eq!(draw.1, "STALEMATE");
    }

    #[test]
    fn replay_reconstructs() {
        // Replay the scholar's-mate event log and confirm terminal state.
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", ""),
            ("MOVE", "a e2 e4 -"),
            ("MOVE", "b e7 e5 -"),
            ("MOVE", "a f1 c4 -"),
            ("MOVE", "b b8 c6 -"),
            ("MOVE", "a d1 h5 -"),
            ("MOVE", "b g8 f6 -"),
            ("MOVE", "a h5 f7 -"),
            ("WINNER", "0"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            Chess::fold(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, Some(0));
        // White queen ended on f7.
        assert_eq!(s.board[sq(5, 6) as usize], Some((Color::White, PieceType::Queen)));
        // Black king still on e8.
        assert_eq!(s.board[sq(4, 7) as usize], Some((Color::Black, PieceType::King)));
    }

    fn join_and_start_blind(payload: &str) -> State {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", payload);
        s
    }

    #[test]
    fn blind_shuffle_populates_facade_for_back_ranks() {
        let s = join_and_start_blind("0 0 1");
        for &sq_idx in &BLIND_SQUARES_WHITE {
            assert!(s.facade.contains_key(&sq_idx));
        }
        for &sq_idx in &BLIND_SQUARES_BLACK {
            assert!(s.facade.contains_key(&sq_idx));
        }
        // King squares stay revealed.
        assert!(!s.facade.contains_key(&sq(4, 0)));
        assert!(!s.facade.contains_key(&sq(4, 7)));
        // Pawns stay revealed.
        for f in 0..8u8 {
            assert!(!s.facade.contains_key(&sq(f, 1)));
            assert!(!s.facade.contains_key(&sq(f, 6)));
        }
    }

    #[test]
    fn blind_shuffle_is_deterministic_for_same_seed() {
        // Drive both games with the same explicit seed token so the
        // shuffle is reproducible.
        let mut s1 = State::default();
        drive(&mut s1, "a", "JOIN", "");
        drive(&mut s1, "b", "JOIN", "");
        Chess::fold(&mut s1, "GAME_STARTED", "0 0 1 12345");
        let mut s2 = State::default();
        drive(&mut s2, "a", "JOIN", "");
        drive(&mut s2, "b", "JOIN", "");
        Chess::fold(&mut s2, "GAME_STARTED", "0 0 1 12345");
        assert_eq!(s1.board, s2.board);
        assert_eq!(s1.facade, s2.facade);
    }

    #[test]
    fn blind_facade_overlay_keeps_displayed_starting_pieces() {
        let s = join_and_start_blind("0 0 1");
        let display = display_board(&s);
        let standard = starting_board();
        assert_eq!(display, standard, "facade overlay must look like the standard starting position");
    }

    #[test]
    fn blind_move_emits_piece_revealed_and_clears_facade() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        Chess::fold(&mut s, "GAME_STARTED", "0 0 1 7");
        let from = sq(1, 0); // b1, facade=Knight
        // Pick a legal display-Knight move (b1 -> c3).
        let to = sq(2, 2);
        let evs = Chess::validate(&s, "a", "MOVE", &format!("{} {}", square_str(from), square_str(to)))
            .expect("validate ok");
        let revealed: Vec<&(String, String)> = evs.iter().filter(|(k, _)| k == "PIECE_REVEALED").collect();
        assert!(!revealed.is_empty(), "first move must reveal");
        for (k, p) in &evs {
            Chess::fold(&mut s, k, p);
        }
        // Source facade is cleared; destination has no facade either.
        assert!(!s.facade.contains_key(&from));
        assert!(!s.facade.contains_key(&to));
    }

    #[test]
    fn blind_replay_reconstructs_post_move_state() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        Chess::fold(&mut s, "GAME_STARTED", "0 0 1 99");
        // Capture the event sequence as the host plays b1 -> c3 (knight facade).
        let from = sq(1, 0);
        let to = sq(2, 2);
        let evs = Chess::validate(&s, "a", "MOVE", &format!("{} {}", square_str(from), square_str(to)))
            .expect("validate ok");
        let mut prefix: Vec<(String, String)> = vec![
            ("PLAYER_JOINED".into(), "a".into()),
            ("PLAYER_JOINED".into(), "b".into()),
            ("GAME_STARTED".into(), "0 0 1 99".into()),
        ];
        prefix.extend(evs.into_iter());
        let mut replay = State::default();
        for (k, p) in &prefix {
            Chess::fold(&mut replay, k, p);
        }
        for (k, p) in &prefix {
            Chess::fold(&mut s, k, p);
        }
        assert_eq!(replay.board, s.board);
        assert_eq!(replay.facade, s.facade);
    }
}
