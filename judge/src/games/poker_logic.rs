// Heads-up no-limit Texas Hold'em as RoomLogic.
//
// Single source of truth: the event log. Hole cards and community cards
// are baked into HAND_STARTED / STREET payloads at validate() time; replay
// reconstructs everything by folding those payloads. There is no RNG path
// during fold.
//
// Per `judge/protocols/poker.md`:
//   - 2 players, dealer button alternates each hand.
//   - Dealer posts small blind, opponent posts big blind.
//   - Pre-flop: dealer acts first. Post-flop: opponent acts first.
//   - GAME_END after configured hand count or first bust.
//   - Followed by WINNER / DRAW so the match-runner's terminal parser
//     keeps working without per-game wiring changes.

use crate::services::room::logic::RoomLogic;
use crate::services::storage::RoomSnapshot;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_PLAYERS: usize = 2;
const DEFAULT_HANDS: u32 = 10;
const DEFAULT_STACK: i64 = 1000;
const DEFAULT_SMALL_BLIND: i64 = 10;

// ---------------------------------------------------------------------------
// Cards
// ---------------------------------------------------------------------------

const RANKS: [char; 13] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
];
const SUITS: [char; 4] = ['c', 'd', 'h', 's'];

/// Cards are encoded as `u8` in `0..52`. `card / 4` is the rank index
/// (0 = "2", 12 = "A"); `card % 4` is the suit index.
fn card_to_str(card: u8) -> String {
    let r = RANKS[(card / 4) as usize];
    let s = SUITS[(card % 4) as usize];
    let mut out = String::with_capacity(2);
    out.push(r);
    out.push(s);
    out
}

fn parse_card(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let rank_ch = bytes[0] as char;
    let suit_ch = bytes[1] as char;
    let r = RANKS.iter().position(|&c| c == rank_ch)? as u8;
    let s = SUITS.iter().position(|&c| c == suit_ch)? as u8;
    Some(r * 4 + s)
}

/// Build a deterministic shuffled deck. Seed from a hash of inputs that
/// the server has at validate-time but cannot leak through fold (we only
/// ever write the resulting cards into events).
fn deal_deck(seed: u64) -> [u8; 52] {
    let mut deck = [0u8; 52];
    for i in 0..52u8 {
        deck[i as usize] = i;
    }
    // xorshift64* — small, deterministic, no external dependency.
    let mut s = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    for i in (1..52).rev() {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        let j = (s % (i as u64 + 1)) as usize;
        deck.swap(i, j);
    }
    deck
}

fn seed_for_hand(players: &[String], hand_no: u32, salt: u64) -> u64 {
    let mut h = DefaultHasher::new();
    "poker".hash(&mut h);
    for p in players {
        p.hash(&mut h);
    }
    hand_no.hash(&mut h);
    salt.hash(&mut h);
    h.finish()
}

// ---------------------------------------------------------------------------
// Hand evaluation: 7-card best-5 ranking returned as a comparable u64.
// Higher is better.
// ---------------------------------------------------------------------------

const CAT_HIGH_CARD: u64 = 0;
const CAT_PAIR: u64 = 1;
const CAT_TWO_PAIR: u64 = 2;
const CAT_TRIPS: u64 = 3;
const CAT_STRAIGHT: u64 = 4;
const CAT_FLUSH: u64 = 5;
const CAT_FULL_HOUSE: u64 = 6;
const CAT_QUADS: u64 = 7;
const CAT_STRAIGHT_FLUSH: u64 = 8;

/// Pack category + up to 5 kickers into a single u64 (4 bits per kicker
/// is enough for ranks 0..=12). Layout, MSB to LSB:
///
/// `[ category : 4 ][ k1 : 4 ][ k2 : 4 ][ k3 : 4 ][ k4 : 4 ][ k5 : 4 ]`
fn pack_rank(cat: u64, k: [u8; 5]) -> u64 {
    (cat << 20)
        | ((k[0] as u64) << 16)
        | ((k[1] as u64) << 12)
        | ((k[2] as u64) << 8)
        | ((k[3] as u64) << 4)
        | (k[4] as u64)
}

/// Detect a straight in `ranks_present` (bitmask of ranks 0..13). Returns
/// the high-card rank of the straight, or None.
fn straight_high(ranks_present: u16) -> Option<u8> {
    // Wheel: A-2-3-4-5 — treat A as low. If ranks 2,3,4,5,A present.
    let wheel_mask: u16 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 12);
    if ranks_present & wheel_mask == wheel_mask {
        // We may still find a higher straight. Continue scanning.
        let mut best: Option<u8> = Some(3); // 5-high straight, top card "5" = idx 3.
        for top in (4..13u8).rev() {
            let mask: u16 = ((1u16 << (top + 1)) - 1) & !((1u16 << (top - 4)) - 1);
            if ranks_present & mask == mask {
                best = Some(top);
                break;
            }
        }
        return best;
    }
    for top in (4..13u8).rev() {
        let mask: u16 = ((1u16 << (top + 1)) - 1) & !((1u16 << (top - 4)) - 1);
        if ranks_present & mask == mask {
            return Some(top);
        }
    }
    None
}

/// Evaluate the best 5-card poker hand from 5..=7 cards.
fn evaluate(cards: &[u8]) -> u64 {
    debug_assert!(cards.len() >= 5 && cards.len() <= 7);

    // Group cards by rank, by suit.
    let mut rank_count = [0u8; 13];
    let mut suit_count = [0u8; 4];
    let mut by_suit_ranks: [u16; 4] = [0; 4]; // ranks present per suit
    for &c in cards {
        let r = (c / 4) as usize;
        let s = (c % 4) as usize;
        rank_count[r] += 1;
        suit_count[s] += 1;
        by_suit_ranks[s] |= 1 << r;
    }
    let mut ranks_present: u16 = 0;
    for r in 0..13 {
        if rank_count[r] > 0 {
            ranks_present |= 1 << r;
        }
    }

    // Straight flush: any suit with >= 5 cards forming a straight.
    for s in 0..4 {
        if suit_count[s] >= 5 {
            if let Some(top) = straight_high(by_suit_ranks[s]) {
                return pack_rank(CAT_STRAIGHT_FLUSH, [top, 0, 0, 0, 0]);
            }
        }
    }

    // Quads.
    let quad = (0..13u8).rev().find(|&r| rank_count[r as usize] == 4);
    if let Some(q) = quad {
        let kicker = (0..13u8)
            .rev()
            .find(|&r| r != q && rank_count[r as usize] >= 1)
            .unwrap_or(0);
        return pack_rank(CAT_QUADS, [q, kicker, 0, 0, 0]);
    }

    // Full house: trips + pair (or trips + trips, use higher trip with lower
    // trip's pair).
    let trips_desc: Vec<u8> = (0..13u8)
        .rev()
        .filter(|&r| rank_count[r as usize] == 3)
        .collect();
    let pairs_desc: Vec<u8> = (0..13u8)
        .rev()
        .filter(|&r| rank_count[r as usize] == 2)
        .collect();
    if !trips_desc.is_empty() {
        let t = trips_desc[0];
        // Best secondary pair: a second trip's rank or the highest pair.
        let secondary = if trips_desc.len() >= 2 {
            Some(trips_desc[1])
        } else {
            pairs_desc.first().copied()
        };
        if let Some(p) = secondary {
            return pack_rank(CAT_FULL_HOUSE, [t, p, 0, 0, 0]);
        }
    }

    // Flush: any suit with >= 5 cards. Use the top 5 ranks.
    for s in 0..4 {
        if suit_count[s] >= 5 {
            let mut top = [0u8; 5];
            let mut filled = 0usize;
            for r in (0..13u8).rev() {
                if by_suit_ranks[s] & (1 << r) != 0 {
                    top[filled] = r;
                    filled += 1;
                    if filled == 5 {
                        break;
                    }
                }
            }
            return pack_rank(CAT_FLUSH, top);
        }
    }

    // Straight.
    if let Some(top) = straight_high(ranks_present) {
        return pack_rank(CAT_STRAIGHT, [top, 0, 0, 0, 0]);
    }

    // Trips (no full house — guarded above).
    if let Some(&t) = trips_desc.first() {
        let mut kickers = [0u8; 5];
        let mut filled = 0usize;
        for r in (0..13u8).rev() {
            if r != t && rank_count[r as usize] >= 1 {
                kickers[filled] = r;
                filled += 1;
                if filled == 2 {
                    break;
                }
            }
        }
        return pack_rank(CAT_TRIPS, [t, kickers[0], kickers[1], 0, 0]);
    }

    // Two pair.
    if pairs_desc.len() >= 2 {
        let p1 = pairs_desc[0];
        let p2 = pairs_desc[1];
        let kicker = (0..13u8)
            .rev()
            .find(|&r| r != p1 && r != p2 && rank_count[r as usize] >= 1)
            .unwrap_or(0);
        return pack_rank(CAT_TWO_PAIR, [p1, p2, kicker, 0, 0]);
    }

    // One pair.
    if let Some(&p) = pairs_desc.first() {
        let mut kickers = [0u8; 5];
        let mut filled = 0usize;
        for r in (0..13u8).rev() {
            if r != p && rank_count[r as usize] >= 1 {
                kickers[filled] = r;
                filled += 1;
                if filled == 3 {
                    break;
                }
            }
        }
        return pack_rank(CAT_PAIR, [p, kickers[0], kickers[1], kickers[2], 0]);
    }

    // High card.
    let mut top = [0u8; 5];
    let mut filled = 0usize;
    for r in (0..13u8).rev() {
        if rank_count[r as usize] >= 1 {
            top[filled] = r;
            filled += 1;
            if filled == 5 {
                break;
            }
        }
    }
    pack_rank(CAT_HIGH_CARD, top)
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
    /// Hand resolved; waiting for next HAND_STARTED.
    Settled,
}

#[derive(Debug, Clone)]
pub struct State {
    pub phase: Phase,
    pub players: Vec<String>,
    pub host: Option<String>,

    pub num_hands: u32,
    pub starting_stack: i64,
    pub small_blind: i64,

    pub stacks: [i64; 2],
    pub hand_no: u32,
    pub dealer: u8,
    pub street: Street,

    pub hole: [Option<(u8, u8)>; 2], // hole cards per player (rank-encoded card)
    pub board: Vec<u8>,              // community cards revealed so far

    /// Per-street current bet to call (max of bet_in_street).
    pub bet_in_street: [i64; 2],
    /// Total this-hand contribution per player (across streets).
    pub committed: [i64; 2],
    pub folded: [bool; 2],
    pub all_in: [bool; 2],
    /// Has each player acted at least once this street since the last raise?
    pub acted_this_street: [bool; 2],
    /// Last raise increment this street (used to compute min raise).
    pub last_raise_size: i64,
    /// Whose turn is it on the current street (player index 0 or 1).
    pub to_act: u8,
    /// Internal salt that bumps every hand to derive a fresh deck seed
    /// without requiring a system source of randomness.
    pub seed_salt: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            num_hands: DEFAULT_HANDS,
            starting_stack: DEFAULT_STACK,
            small_blind: DEFAULT_SMALL_BLIND,
            stacks: [DEFAULT_STACK, DEFAULT_STACK],
            hand_no: 0,
            dealer: 0,
            street: Street::Settled,
            hole: [None, None],
            board: Vec::new(),
            bet_in_street: [0, 0],
            committed: [0, 0],
            folded: [false, false],
            all_in: [false, false],
            acted_this_street: [false, false],
            last_raise_size: 0,
            to_act: 0,
            seed_salt: 0,
        }
    }
}

impl State {
    fn player_index(&self, pid: &str) -> Option<u8> {
        self.players.iter().position(|p| p == pid).map(|i| i as u8)
    }
    fn pot(&self) -> i64 {
        self.committed[0] + self.committed[1]
    }
    fn bb(&self) -> i64 {
        self.small_blind * 2
    }
    fn max_bet(&self) -> i64 {
        self.bet_in_street[0].max(self.bet_in_street[1])
    }
    fn first_to_act_postflop(&self) -> u8 {
        // Heads-up: dealer acts second post-flop, so opponent of dealer.
        1 - self.dealer
    }
    fn first_to_act_preflop(&self) -> u8 {
        // Heads-up: dealer (small blind) acts first preflop.
        self.dealer
    }
}

// ---------------------------------------------------------------------------
// Logic
// ---------------------------------------------------------------------------

pub struct Poker;

/// Parameters from a START payload. Returns defaults on parse failure.
fn parse_start_params(payload: &str) -> (u32, i64, i64) {
    let parts: Vec<&str> = payload.split_whitespace().collect();
    let num_hands = parts.first().and_then(|s| s.parse::<u32>().ok()).unwrap_or(DEFAULT_HANDS);
    let stack = parts.get(1).and_then(|s| s.parse::<i64>().ok()).unwrap_or(DEFAULT_STACK);
    let sb = parts.get(2).and_then(|s| s.parse::<i64>().ok()).unwrap_or(DEFAULT_SMALL_BLIND);
    (num_hands.max(1), stack.max(2), sb.max(1))
}

/// Output of one validated action prior to events being assembled.
struct ActionResult {
    action_event: (String, String),
}

impl Poker {
    /// Build a HAND_STARTED event payload from current state.
    fn build_hand_started(state: &State) -> (String, String) {
        let next_hand_no = state.hand_no + 1;
        let next_dealer = if state.hand_no == 0 { 0u8 } else { 1 - state.dealer };
        let salt = state.seed_salt.wrapping_add(next_hand_no as u64);
        let seed = seed_for_hand(&state.players, next_hand_no, salt);
        let deck = deal_deck(seed);
        // Layout: hole0a, hole0b, hole1a, hole1b, burn, flop1, flop2, flop3, burn, turn, burn, river.
        // We only emit hole cards in HAND_STARTED. The rest comes via STREET later.
        let payload = format!(
            "{} {} {} {} {} {}",
            next_hand_no,
            next_dealer,
            card_to_str(deck[0]),
            card_to_str(deck[1]),
            card_to_str(deck[2]),
            card_to_str(deck[3]),
        );
        ("HAND_STARTED".into(), payload)
    }

    /// Build STREET event for the requested phase based on state. Reuses
    /// the same deck the HAND_STARTED used (same seed inputs).
    fn build_street(state: &State, street: Street) -> Option<(String, String)> {
        let salt = state.seed_salt; // already includes hand_no shift from start_hand
        let seed = seed_for_hand(&state.players, state.hand_no, salt);
        let deck = deal_deck(seed);
        // Indices after 4 hole cards: burn(4), flop(5,6,7), burn(8), turn(9), burn(10), river(11).
        match street {
            Street::Flop => Some((
                "STREET".into(),
                format!(
                    "flop {} {} {}",
                    card_to_str(deck[5]),
                    card_to_str(deck[6]),
                    card_to_str(deck[7]),
                ),
            )),
            Street::Turn => Some(("STREET".into(), format!("turn {}", card_to_str(deck[9])))),
            Street::River => {
                Some(("STREET".into(), format!("river {}", card_to_str(deck[11]))))
            }
            _ => None,
        }
    }

    /// After a player action, determine whether the betting round closes
    /// and what events follow (additional STREET, HAND_END, GAME_END).
    fn resolve_after_action(state: &State) -> Vec<(String, String)> {
        // Walk a scratch copy forward to the natural break point.
        let mut s = state.clone();
        let mut out: Vec<(String, String)> = Vec::new();

        // Loop bound: at most 4 streets * 2 transitions + game end = small constant.
        // Use a hard cap to satisfy the unbounded-loop policy.
        for _ in 0..16 {
            // Hand resolved by fold?
            if s.folded[0] || s.folded[1] {
                let winner = if s.folded[0] { 1u8 } else { 0u8 };
                out.extend(Self::settle_hand(&mut s, Some(winner), "FOLD"));
                break;
            }

            // Round closed?
            let round_closed = Self::round_closed(&s);
            if !round_closed {
                break;
            }

            // Reset bets, decide next street.
            s.bet_in_street = [0, 0];
            s.acted_this_street = [false, false];
            s.last_raise_size = 0;

            let next_street = match s.street {
                Street::Preflop => Street::Flop,
                Street::Flop => Street::Turn,
                Street::Turn => Street::River,
                Street::River => {
                    // Showdown.
                    out.extend(Self::settle_showdown(&mut s));
                    break;
                }
                Street::Settled => break,
            };
            s.street = next_street;

            // Emit STREET event with cards.
            if let Some(ev) = Self::build_street(&s, next_street) {
                // Apply community cards into scratch state for evaluation.
                let parts: Vec<&str> = ev.1.split_whitespace().collect();
                for tok in parts.iter().skip(1) {
                    if let Some(c) = parse_card(tok) {
                        s.board.push(c);
                    }
                }
                out.push(ev);
            }
            out.push(("POT".into(), s.pot().to_string()));

            // If both all-in, automatically run remaining streets to showdown.
            // Loop continues since round_closed will be true again next iter.
            if s.all_in[0] && s.all_in[1] {
                continue;
            }

            // Set first to act on the new street.
            s.to_act = s.first_to_act_postflop();
            // Skip if first-to-act is all-in — opponent has nothing to do, loop again.
            if s.all_in[s.to_act as usize] && s.all_in[(1 - s.to_act) as usize] {
                continue;
            }
            break;
        }

        out
    }

    /// A street's betting round is closed if both players have matched
    /// the highest bet AND each has had a chance to act, OR if at least
    /// one is all-in and the other has matched/folded.
    fn round_closed(s: &State) -> bool {
        if s.folded[0] || s.folded[1] {
            return true;
        }
        let max = s.max_bet();
        let both_matched = s.bet_in_street[0] == max && s.bet_in_street[1] == max;
        let both_acted = s.acted_this_street[0] && s.acted_this_street[1];

        if s.all_in[0] && s.all_in[1] {
            return true;
        }
        if s.all_in[0] || s.all_in[1] {
            // Acting player must have matched (or folded, which exits earlier).
            return both_matched;
        }

        both_matched && both_acted
    }

    /// Settle a hand that ended by fold. Updates stacks and emits HAND_END.
    fn settle_hand(
        s: &mut State,
        winner: Option<u8>,
        reason: &str,
    ) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let pot = s.pot();
        let winner_idx_str: String = match winner {
            Some(w) => {
                s.stacks[w as usize] += pot;
                w.to_string()
            }
            None => {
                // Split.
                let half = pot / 2;
                s.stacks[0] += half;
                s.stacks[1] += pot - half;
                "-1".to_string()
            }
        };
        out.push((
            "HAND_END".into(),
            format!(
                "{} {} {} {}",
                winner_idx_str, s.stacks[0], s.stacks[1], reason
            ),
        ));
        // Mark hand done in scratch state.
        s.street = Street::Settled;
        s.committed = [0, 0];
        s.bet_in_street = [0, 0];

        // Game end?
        if s.stacks[0] == 0 || s.stacks[1] == 0 || s.hand_no >= s.num_hands {
            out.extend(Self::game_end_events(s));
            return out;
        }

        // Otherwise queue next hand: HAND_STARTED for hand_no+1.
        out.push(Self::build_hand_started(s));
        Self::collect_blinds_after_start(&mut out, s);
        out
    }

    /// Settle a hand that reached showdown.
    fn settle_showdown(s: &mut State) -> Vec<(String, String)> {
        // Evaluate each player's 7-card hand.
        let mut sets: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
        for i in 0..2usize {
            if let Some((a, b)) = s.hole[i] {
                sets[i].push(a);
                sets[i].push(b);
            }
            sets[i].extend_from_slice(&s.board);
        }
        let r0 = if sets[0].len() >= 5 { evaluate(&sets[0]) } else { 0 };
        let r1 = if sets[1].len() >= 5 { evaluate(&sets[1]) } else { 0 };
        let (winner, reason) = if r0 > r1 {
            (Some(0u8), "SHOWDOWN")
        } else if r1 > r0 {
            (Some(1u8), "SHOWDOWN")
        } else {
            (None, "SPLIT")
        };
        Self::settle_hand(s, winner, reason)
    }

    /// Build GAME_END + WINNER/DRAW backwards-compat events. Updates
    /// scratch phase to Finished.
    fn game_end_events(s: &mut State) -> Vec<(String, String)> {
        s.phase = Phase::Finished;
        let mut out = vec![(
            "GAME_END".into(),
            format!("{} {}", s.stacks[0], s.stacks[1]),
        )];
        if s.stacks[0] > s.stacks[1] {
            out.push(("WINNER".into(), "0".into()));
        } else if s.stacks[1] > s.stacks[0] {
            out.push(("WINNER".into(), "1".into()));
        } else {
            // Tiebreaker per spec: dealer of the last hand wins.
            // For DRAW back-compat scoring, emit DRAW. The dealer-tiebreak
            // is purely informational and not encoded in WINNER, since the
            // existing scorer treats DRAW as zero-zero.
            out.push(("DRAW".into(), String::new()));
        }
        out
    }

    /// Push the START_HAND companion events: blinds (recorded as ACTION
    /// posts) and the initial POT total.
    fn collect_blinds_after_start(out: &mut Vec<(String, String)>, s: &mut State) {
        // We mutate `s` to reflect the freshly-started hand so that subsequent
        // event construction (e.g. resolve_after_action) reads the right state.
        // The same mutations will be replayed by `fold` when these events
        // are committed.
        let prev_hand_no = s.hand_no;
        let next_hand_no = prev_hand_no + 1;
        let next_dealer = if prev_hand_no == 0 { 0u8 } else { 1 - s.dealer };

        s.hand_no = next_hand_no;
        s.dealer = next_dealer;
        s.seed_salt = s.seed_salt.wrapping_add(next_hand_no as u64);
        s.street = Street::Preflop;
        s.board.clear();
        s.bet_in_street = [0, 0];
        s.committed = [0, 0];
        s.folded = [false, false];
        s.all_in = [false, false];
        s.acted_this_street = [false, false];
        s.last_raise_size = 0;

        // Re-derive hole cards from the same seed used by HAND_STARTED.
        let seed = seed_for_hand(&s.players, next_hand_no, s.seed_salt);
        let deck = deal_deck(seed);
        s.hole[0] = Some((deck[0], deck[1]));
        s.hole[1] = Some((deck[2], deck[3]));

        // Post blinds: dealer = SB, opponent = BB.
        let sb = s.small_blind.min(s.stacks[next_dealer as usize]);
        let bb_player = 1 - next_dealer;
        let bb_amt = s.bb().min(s.stacks[bb_player as usize]);
        s.stacks[next_dealer as usize] -= sb;
        s.stacks[bb_player as usize] -= bb_amt;
        s.committed[next_dealer as usize] = sb;
        s.committed[bb_player as usize] = bb_amt;
        s.bet_in_street[next_dealer as usize] = sb;
        s.bet_in_street[bb_player as usize] = bb_amt;
        s.last_raise_size = s.bb();
        if s.stacks[next_dealer as usize] == 0 {
            s.all_in[next_dealer as usize] = true;
        }
        if s.stacks[bb_player as usize] == 0 {
            s.all_in[bb_player as usize] = true;
        }
        s.to_act = next_dealer; // dealer acts first preflop in heads-up.

        out.push((
            "ACTION".into(),
            format!("{} POST {}", s.players[next_dealer as usize], sb),
        ));
        out.push((
            "ACTION".into(),
            format!("{} POST {}", s.players[bb_player as usize], bb_amt),
        ));
        out.push(("POT".into(), s.pot().to_string()));
    }

    /// Validate a poker action and return the ACTION event + a flag.
    fn validate_poker_action(
        state: &State,
        idx: u8,
        kind: &str,
        payload: &str,
    ) -> Result<ActionResult, String> {
        if state.phase != Phase::Playing {
            return Err("not playing".into());
        }
        if state.street == Street::Settled {
            return Err("between hands".into());
        }
        if state.to_act != idx {
            return Err("not your turn".into());
        }
        if state.folded[idx as usize] {
            return Err("you have folded".into());
        }
        if state.all_in[idx as usize] {
            return Err("you are all-in".into());
        }

        let pid = &state.players[idx as usize];
        let max_bet = state.max_bet();
        let to_call = max_bet - state.bet_in_street[idx as usize];
        let stack = state.stacks[idx as usize];

        match kind {
            "FOLD" => {
                if to_call == 0 {
                    // Folding when you can check is legal but pointless; allow it.
                }
                Ok(ActionResult {
                    action_event: ("ACTION".into(), format!("{pid} FOLD")),
                })
            }
            "CHECK" => {
                if to_call != 0 {
                    return Err("cannot check, there is a bet to call".into());
                }
                Ok(ActionResult {
                    action_event: ("ACTION".into(), format!("{pid} CHECK")),
                })
            }
            "CALL" => {
                if to_call == 0 {
                    return Err("nothing to call".into());
                }
                let pay = to_call.min(stack);
                Ok(ActionResult {
                    action_event: ("ACTION".into(), format!("{pid} CALL {pay}")),
                })
            }
            "BET" => {
                if max_bet != 0 {
                    return Err("there is already a bet; use RAISE".into());
                }
                let amt: i64 = payload
                    .trim()
                    .parse()
                    .map_err(|_| "bad bet amount".to_string())?;
                if amt < state.bb() {
                    return Err("bet must be at least the big blind".into());
                }
                if amt > stack {
                    return Err("bet exceeds stack".into());
                }
                Ok(ActionResult {
                    action_event: ("ACTION".into(), format!("{pid} BET {amt}")),
                })
            }
            "RAISE" => {
                if max_bet == 0 {
                    return Err("nothing to raise; use BET".into());
                }
                let to_amt: i64 = payload
                    .trim()
                    .parse()
                    .map_err(|_| "bad raise amount".to_string())?;
                let increment = to_amt - max_bet;
                let min_increment = state.last_raise_size.max(state.bb());
                if increment < min_increment {
                    return Err(format!(
                        "raise must be at least to {}",
                        max_bet + min_increment
                    ));
                }
                let need = to_amt - state.bet_in_street[idx as usize];
                if need > stack {
                    return Err("raise exceeds stack; use ALLIN".into());
                }
                Ok(ActionResult {
                    action_event: ("ACTION".into(), format!("{pid} RAISE {to_amt}")),
                })
            }
            "ALLIN" => {
                if stack == 0 {
                    return Err("no chips to push".into());
                }
                Ok(ActionResult {
                    action_event: ("ACTION".into(), format!("{pid} ALLIN {stack}")),
                })
            }
            _ => Err(format!("unknown poker action: {kind}")),
        }
    }
}

impl RoomLogic for Poker {
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
                let (n, stack, sb) = parse_start_params(payload);
                state.num_hands = n;
                state.starting_stack = stack;
                state.small_blind = sb;
                state.stacks = [stack, stack];
                state.phase = Phase::Playing;
                state.hand_no = 0;
                state.dealer = 0; // will flip to 0 on first hand_started anyway
                state.street = Street::Settled;
                state.seed_salt = 0;
            }
            "HAND_STARTED" => {
                // payload: "<hand_no> <dealer_idx> <hole0a> <hole0b> <hole1a> <hole1b>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() < 6 {
                    return;
                }
                let hand_no: u32 = parts[0].parse().unwrap_or(0);
                let dealer: u8 = parts[1].parse().unwrap_or(0);
                let h0a = parse_card(parts[2]);
                let h0b = parse_card(parts[3]);
                let h1a = parse_card(parts[4]);
                let h1b = parse_card(parts[5]);

                state.hand_no = hand_no;
                state.dealer = dealer;
                state.seed_salt = state.seed_salt.wrapping_add(hand_no as u64);
                state.street = Street::Preflop;
                state.board.clear();
                state.bet_in_street = [0, 0];
                state.committed = [0, 0];
                state.folded = [false, false];
                state.all_in = [false, false];
                state.acted_this_street = [false, false];
                state.last_raise_size = 0;
                if let (Some(a), Some(b)) = (h0a, h0b) {
                    state.hole[0] = Some((a, b));
                }
                if let (Some(a), Some(b)) = (h1a, h1b) {
                    state.hole[1] = Some((a, b));
                }
                state.to_act = dealer; // dealer acts first preflop heads-up
            }
            "STREET" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.is_empty() {
                    return;
                }
                let next = match parts[0] {
                    "flop" => Street::Flop,
                    "turn" => Street::Turn,
                    "river" => Street::River,
                    _ => return,
                };
                state.street = next;
                for tok in parts.iter().skip(1) {
                    if let Some(c) = parse_card(tok) {
                        state.board.push(c);
                    }
                }
                state.bet_in_street = [0, 0];
                state.acted_this_street = [false, false];
                state.last_raise_size = 0;
                state.to_act = state.first_to_act_postflop();
            }
            "ACTION" => {
                // payload: "<pid> <verb> [amount]"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() < 2 {
                    return;
                }
                let pid = parts[0];
                let verb = parts[1];
                let amount: i64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                let Some(idx) = state.player_index(pid) else {
                    return;
                };
                let i = idx as usize;
                match verb {
                    "POST" => {
                        // Posted by start-hand mechanics; nothing extra to do —
                        // hole/blinds were already initialized in HAND_STARTED
                        // fold path. We simulate the same here so the fold of
                        // ACTION POST does not double-deduct: only deduct if
                        // committed[i] is still zero (state was rebuilt cleanly).
                        if state.committed[i] == 0 {
                            let pay = amount.min(state.stacks[i]);
                            state.stacks[i] -= pay;
                            state.committed[i] = pay;
                            state.bet_in_street[i] = pay;
                            if state.stacks[i] == 0 {
                                state.all_in[i] = true;
                            }
                            if state.bet_in_street[i] > state.last_raise_size {
                                state.last_raise_size = state.bb();
                            }
                            // Toggle to_act to dealer (preflop-first); recompute.
                            state.to_act = state.first_to_act_preflop();
                        }
                    }
                    "FOLD" => {
                        state.folded[i] = true;
                        state.acted_this_street[i] = true;
                    }
                    "CHECK" => {
                        state.acted_this_street[i] = true;
                        state.to_act = 1 - idx;
                    }
                    "CALL" => {
                        let pay = amount.min(state.stacks[i]);
                        state.stacks[i] -= pay;
                        state.committed[i] += pay;
                        state.bet_in_street[i] += pay;
                        if state.stacks[i] == 0 {
                            state.all_in[i] = true;
                        }
                        state.acted_this_street[i] = true;
                        state.to_act = 1 - idx;
                    }
                    "BET" => {
                        let pay = amount.min(state.stacks[i]);
                        state.stacks[i] -= pay;
                        state.committed[i] += pay;
                        state.bet_in_street[i] += pay;
                        if state.stacks[i] == 0 {
                            state.all_in[i] = true;
                        }
                        state.last_raise_size = pay;
                        // A bet reopens action.
                        state.acted_this_street = [false, false];
                        state.acted_this_street[i] = true;
                        state.to_act = 1 - idx;
                    }
                    "RAISE" => {
                        let to_amt = amount;
                        let need = (to_amt - state.bet_in_street[i]).max(0);
                        let pay = need.min(state.stacks[i]);
                        let prev_max = state.max_bet();
                        state.stacks[i] -= pay;
                        state.committed[i] += pay;
                        state.bet_in_street[i] += pay;
                        let new_max = state.bet_in_street[i];
                        let increment = new_max - prev_max;
                        if increment >= state.last_raise_size {
                            state.last_raise_size = increment;
                            state.acted_this_street = [false, false];
                        }
                        if state.stacks[i] == 0 {
                            state.all_in[i] = true;
                        }
                        state.acted_this_street[i] = true;
                        state.to_act = 1 - idx;
                    }
                    "ALLIN" => {
                        let pay = amount.min(state.stacks[i]);
                        let prev_max = state.max_bet();
                        state.stacks[i] -= pay;
                        state.committed[i] += pay;
                        state.bet_in_street[i] += pay;
                        let new_max = state.bet_in_street[i];
                        let increment = new_max - prev_max;
                        if increment > 0 && increment >= state.last_raise_size {
                            state.last_raise_size = increment;
                            state.acted_this_street = [false, false];
                        }
                        state.all_in[i] = true;
                        state.acted_this_street[i] = true;
                        state.to_act = 1 - idx;
                    }
                    _ => {}
                }
            }
            "POT" => {
                // Snapshot only — pot is derived from `committed`.
            }
            "HAND_END" => {
                // payload: "<winner|-1> <chips0> <chips1> <reason>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() < 3 {
                    return;
                }
                if let (Ok(c0), Ok(c1)) = (parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                    state.stacks = [c0, c1];
                }
                state.street = Street::Settled;
                state.committed = [0, 0];
                state.bet_in_street = [0, 0];
                state.folded = [false, false];
                state.all_in = [false, false];
                state.acted_this_street = [false, false];
                state.board.clear();
                state.last_raise_size = 0;
            }
            "GAME_END" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == 2 {
                    if let (Ok(c0), Ok(c1)) =
                        (parts[0].parse::<i64>(), parts[1].parse::<i64>())
                    {
                        state.stacks = [c0, c1];
                    }
                }
                state.phase = Phase::Finished;
            }
            "WINNER" | "DRAW" => {
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
                    if let Some(h) = state
                        .players
                        .iter()
                        .find(|p| p.as_str() != player)
                        .cloned()
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
                let (n, stack, sb) = parse_start_params(payload);
                let mut out: Vec<(String, String)> =
                    vec![("GAME_STARTED".into(), format!("{n} {stack} {sb}"))];

                // Build a scratch state matching what `fold` will produce
                // for GAME_STARTED, then add the first hand's events.
                let mut scratch = state.clone();
                scratch.num_hands = n;
                scratch.starting_stack = stack;
                scratch.small_blind = sb;
                scratch.stacks = [stack, stack];
                scratch.phase = Phase::Playing;
                scratch.hand_no = 0;
                scratch.dealer = 0;
                scratch.street = Street::Settled;
                scratch.seed_salt = 0;

                out.push(Self::build_hand_started(&scratch));
                Self::collect_blinds_after_start(&mut out, &mut scratch);
                Ok(out)
            }
            // Poker actions:
            kind @ ("FOLD" | "CHECK" | "CALL" | "BET" | "RAISE" | "ALLIN") => {
                let idx = state
                    .player_index(player)
                    .ok_or_else(|| "not in room".to_string())?;
                let res = Self::validate_poker_action(state, idx, kind, payload)?;
                let mut out = vec![res.action_event.clone()];

                // Apply action to scratch state, then resolve onward events.
                let mut scratch = state.clone();
                Poker::fold(&mut scratch, &res.action_event.0, &res.action_event.1);
                out.push(("POT".into(), scratch.pot().to_string()));
                let extra = Self::resolve_after_action(&scratch);
                out.extend(extra);
                Ok(out)
            }
            _ => Err(format!("unknown action: {kind}")),
        }
    }

    fn max_players() -> usize {
        MAX_PLAYERS
    }

    fn game_id() -> &'static str {
        "poker"
    }

    fn pending_players(state: &Self::State) -> Vec<String> {
        if state.phase != Phase::Playing {
            return Vec::new();
        }
        if state.street == Street::Settled {
            return Vec::new();
        }
        let i = state.to_act as usize;
        if state.folded[i] || state.all_in[i] {
            return Vec::new();
        }
        state.players.get(i).cloned().into_iter().collect()
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn drive(s: &mut State, player: &str, kind: &str, payload: &str) -> Vec<(String, String)> {
        let evs = Poker::validate(s, player, kind, payload).expect("validate ok");
        for (k, p) in &evs {
            Poker::fold(s, k, p);
        }
        evs
    }

    fn cards(strs: &[&str]) -> Vec<u8> {
        strs.iter().map(|s| parse_card(s).unwrap()).collect()
    }

    #[test]
    fn round_trip_card_codec() {
        for c in 0..52u8 {
            let s = card_to_str(c);
            assert_eq!(parse_card(&s), Some(c));
        }
    }

    #[test]
    fn straight_flush_beats_quads() {
        let sf = cards(&["2c", "3c", "4c", "5c", "6c", "Ah", "As"]);
        let q = cards(&["Ah", "Ac", "Ad", "As", "Kh", "Kd", "Qs"]);
        assert!(evaluate(&sf) > evaluate(&q));
    }

    #[test]
    fn quads_beat_full_house() {
        let q = cards(&["Ah", "Ac", "Ad", "As", "5h", "5d", "2c"]);
        let fh = cards(&["Kh", "Kc", "Kd", "5s", "5h", "2d", "3c"]);
        assert!(evaluate(&q) > evaluate(&fh));
    }

    #[test]
    fn full_house_beats_flush() {
        let fh = cards(&["Kh", "Kc", "Kd", "5s", "5h", "2d", "3c"]);
        let fl = cards(&["2c", "5c", "7c", "9c", "Jc", "Ah", "Ks"]);
        assert!(evaluate(&fh) > evaluate(&fl));
    }

    #[test]
    fn flush_beats_straight() {
        let fl = cards(&["2c", "5c", "7c", "9c", "Jc", "Ah", "Ks"]);
        let st = cards(&["5d", "6c", "7s", "8h", "9d", "2c", "3c"]);
        assert!(evaluate(&fl) > evaluate(&st));
    }

    #[test]
    fn wheel_straight_recognised() {
        let wheel = cards(&["Ah", "2c", "3d", "4s", "5h", "Ks", "Qc"]);
        let r = evaluate(&wheel);
        let cat = (r >> 20) & 0xF;
        assert_eq!(cat, CAT_STRAIGHT);
        // Top card should be index of "5" = 3.
        let high = (r >> 16) & 0xF;
        assert_eq!(high, 3);
    }

    #[test]
    fn lobby_join_and_start() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 200 10");
        assert_eq!(s.phase, Phase::Playing);
        assert_eq!(s.num_hands, 1);
        assert_eq!(s.starting_stack, 200);
        assert_eq!(s.small_blind, 10);
        // Blinds posted: dealer (a, idx 0) SB=10, BB=20.
        assert_eq!(s.committed, [10, 20]);
        assert_eq!(s.stacks, [190, 180]);
        assert_eq!(s.to_act, 0); // dealer acts first preflop
        assert_eq!(s.hand_no, 1);
    }

    #[test]
    fn fold_loses_pot() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 200 10");
        // a is dealer (sb=10), b is bb=20. Pot=30.
        // a folds preflop. b wins 30, but b only contributed 20 — so b nets +10.
        drive(&mut s, "a", "FOLD", "");
        assert_eq!(s.phase, Phase::Finished); // num_hands=1, hand done -> game end
        // a started 200, lost 10 sb -> 190.
        // b started 200, posted 20, gained pot 30 -> 200 - 20 + 30 = 210.
        assert_eq!(s.stacks, [190, 210]);
    }

    #[test]
    fn check_check_advances_streets() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 1000 10");
        // Preflop: dealer (a) calls 10 to match BB. b then checks.
        drive(&mut s, "a", "CALL", "");
        assert_eq!(s.bet_in_street, [20, 20]);
        assert_eq!(s.to_act, 1);
        drive(&mut s, "b", "CHECK", "");
        // Flop dealt — opponent (idx 1) acts first postflop.
        assert_eq!(s.street, Street::Flop);
        assert_eq!(s.board.len(), 3);
        assert_eq!(s.to_act, 1);
        // b checks, a checks → turn.
        drive(&mut s, "b", "CHECK", "");
        drive(&mut s, "a", "CHECK", "");
        assert_eq!(s.street, Street::Turn);
        assert_eq!(s.board.len(), 4);
        // turn checks → river.
        drive(&mut s, "b", "CHECK", "");
        drive(&mut s, "a", "CHECK", "");
        assert_eq!(s.street, Street::River);
        assert_eq!(s.board.len(), 5);
        // river checks → showdown → game end (1-hand match).
        drive(&mut s, "b", "CHECK", "");
        drive(&mut s, "a", "CHECK", "");
        assert_eq!(s.phase, Phase::Finished);
        // Stacks must sum to 2000.
        assert_eq!(s.stacks[0] + s.stacks[1], 2000);
    }

    #[test]
    fn allin_runs_streets_to_showdown() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 100 10");
        // a (dealer, sb=10, stack=90), b (bb=20, stack=80). Pot=30.
        // a goes ALLIN for 90 -> bet_in_street[a] = 100.
        drive(&mut s, "a", "ALLIN", "");
        assert!(s.all_in[0]);
        assert_eq!(s.to_act, 1);
        // b calls (puts in 80 more).
        drive(&mut s, "b", "CALL", "");
        // After call b has 0 stack -> all-in.
        // Streets auto-deal flop/turn/river then showdown -> game end.
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.stacks[0] + s.stacks[1], 200);
    }

    #[test]
    fn replay_reconstructs() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 200 10");
        // Capture all events that have been recorded via fold so far.
        // Run a fold/fold game by having 'a' fold preflop.
        drive(&mut s, "a", "FOLD", "");

        // Now pretend we replay the same sequence into a fresh state by
        // recording the events manually. Replay must yield equivalent
        // observable state (phase, stacks, num_hands).
        let scripted: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", "1 200 10"),
            // The hand events are deterministic given the player names + hand_no.
            // We reconstruct them by walking validate again from scratch.
        ];
        let mut fresh = State::default();
        for (k, p) in &scripted {
            Poker::fold(&mut fresh, k, p);
        }
        // Re-derive HAND_STARTED via validate's helper to keep card pool
        // identical.
        let hand_started = Poker::build_hand_started(&fresh);
        Poker::fold(&mut fresh, &hand_started.0, &hand_started.1);
        // Posts.
        let mut blinds = Vec::new();
        Poker::collect_blinds_after_start(&mut blinds, &mut fresh);
        // The collect_blinds_after_start helper has already mutated `fresh`
        // by design (validate's scratch path). For replay, rebuild fresh
        // using only fold over the (HAND_STARTED + ACTION POST x2) events.
        let mut fresh2 = State::default();
        for (k, p) in &scripted {
            Poker::fold(&mut fresh2, k, p);
        }
        Poker::fold(&mut fresh2, &hand_started.0, &hand_started.1);
        for (k, p) in &blinds {
            Poker::fold(&mut fresh2, k, p);
        }
        // Apply the fold action and its consequent events.
        Poker::fold(&mut fresh2, "ACTION", "a FOLD");
        Poker::fold(
            &mut fresh2,
            "HAND_END",
            &format!("1 {} {} FOLD", 190, 210),
        );
        Poker::fold(&mut fresh2, "GAME_END", "190 210");
        Poker::fold(&mut fresh2, "WINNER", "1");
        assert_eq!(fresh2.phase, Phase::Finished);
        assert_eq!(fresh2.stacks, [190, 210]);

        // Also assert original drive run matches.
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.stacks, [190, 210]);
    }

    #[test]
    fn cannot_act_off_turn() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 200 10");
        // 'b' is not to act preflop (dealer 'a' is). Should be rejected.
        assert!(Poker::validate(&s, "b", "CHECK", "").is_err());
    }

    #[test]
    fn check_illegal_when_bet_outstanding() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1 200 10");
        // Dealer faces BB=20 vs SB=10 -> to_call=10. CHECK invalid.
        assert!(Poker::validate(&s, "a", "CHECK", "").is_err());
    }
}
