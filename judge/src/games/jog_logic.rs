// Jar of greed as RoomLogic.
//
// 2..=8 players. Each round every player secretly contributes coins to a
// shared jar; once everyone has contributed, the jar is multiplied
// (fixed factor > 1, or randomized per round) and split evenly between
// all players. All coin arithmetic floors fractional results. Highest
// stack at the end of the configured round count wins; final balances
// are the per-player score.
//
// Per-player coin balance visibility is controlled by the `blind`
// configuration flag (defaults to true).
//
//   * `blind = true` — `ROUND_RESULT` carries only the aggregate (jar,
//     multiplier, payout); per-player balances are hidden until
//     `GAME_END`. Each player still knows their own balance because
//     they emitted their own CONTRIBUTE and they receive the payout.
//   * `blind = false` — `ROUND_RESULT` appends the per-player balance
//     tail `<c0> <c1> ... <cN-1>` so spectators/replay viewers can see
//     each player's running stack after every payout.
//
// In either mode contribution *amounts* during the contributing phase
// stay hidden at the UI layer: CONTRIBUTE events carry `<pid>
// <amount>` so the log replays deterministically, but the human web
// client must only render its own amount until the round resolves.
// This matches the project's existing event-sourced redaction posture
// (see poker hole cards).
//
// CONTRIBUTE events still carry `<pid> <amount>` so the log replays
// deterministically. The "amount stays secret in real time" guarantee
// is therefore enforced by the human web client (display only own
// amount + own running balance), the same way poker hole cards are
// client-redacted. For bot tournaments where every player is a
// subprocess and the log is the audit trail, both bots will see each
// other's amounts; this matches the project's existing event-sourced
// redaction posture.
//
// Player roster is locked at GAME_STARTED — `started_players` snapshots
// the join-order list and per-player arrays (`coins`, `pending_contrib`)
// are sized to it. A LEAVE during play removes the player from the
// lobby list but does not resize the score arrays. A player who left
// can no longer contribute and the round can no longer resolve, which
// the timeout watcher will terminate as a forfeit.

use crate::services::room::logic::RoomLogic;
use crate::services::storage::RoomSnapshot;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_PLAYERS: usize = 8;
const MIN_PLAYERS_TO_START: usize = 2;

pub const DEFAULT_STARTING_COINS: i64 = 10;
pub const DEFAULT_ROUNDS: u32 = 5;
pub const DEFAULT_MULTIPLIER: f64 = 2.0;

const MIN_STARTING_COINS: i64 = 1;
const MAX_STARTING_COINS: i64 = 1_000_000;
const MIN_ROUNDS: u32 = 1;
const MAX_ROUNDS: u32 = 1000;

const MIN_MULTIPLIER: f64 = 1.01;
const MAX_MULTIPLIER: f64 = 10.0;

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
    pub starting_coins: i64,
    pub total_rounds: u32,
    pub multiplier: f64,
    pub random: bool,
    /// When true (default), per-player balances are hidden from
    /// `ROUND_RESULT` until `GAME_END`. When false, every
    /// `ROUND_RESULT` carries the full balance tail.
    pub blind: bool,
    pub current_round: u32,
    pub started_players: Vec<String>,
    /// Authoritative per-player coin balances (server-side state).
    /// NOT broadcast verbatim during play — only revealed in GAME_END.
    pub coins: Vec<i64>,
    pub pending_contrib: Vec<Option<i64>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            starting_coins: DEFAULT_STARTING_COINS,
            total_rounds: DEFAULT_ROUNDS,
            multiplier: DEFAULT_MULTIPLIER,
            random: false,
            blind: true,
            current_round: 0,
            started_players: Vec::new(),
            coins: Vec::new(),
            pending_contrib: Vec::new(),
        }
    }
}

impl State {
    fn lobby_index(&self, pid: &str) -> Option<usize> {
        self.players.iter().position(|p| p == pid)
    }
    fn play_index(&self, pid: &str) -> Option<usize> {
        self.started_players.iter().position(|p| p == pid)
    }
}

fn parse_start_params(payload: &str) -> (i64, u32, bool, f64, bool) {
    let parts: Vec<&str> = payload.split_whitespace().collect();
    let coins = parts
        .first()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(DEFAULT_STARTING_COINS)
        .clamp(MIN_STARTING_COINS, MAX_STARTING_COINS);
    let rounds = parts
        .get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(DEFAULT_ROUNDS)
        .clamp(MIN_ROUNDS, MAX_ROUNDS);
    let random = parts
        .get(2)
        .and_then(|s| s.parse::<u32>().ok())
        .map(|n| n != 0)
        .unwrap_or(false);
    let mult = parts
        .get(3)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(DEFAULT_MULTIPLIER)
        .clamp(MIN_MULTIPLIER, MAX_MULTIPLIER);
    let blind = parts
        .get(4)
        .and_then(|s| s.parse::<u32>().ok())
        .map(|n| n != 0)
        .unwrap_or(true);
    (coins, rounds, random, mult, blind)
}

fn random_multiplier(players: &[String], round: u32, max: f64) -> f64 {
    let mut h = DefaultHasher::new();
    "jar-of-greed".hash(&mut h);
    for p in players {
        p.hash(&mut h);
    }
    round.hash(&mut h);
    let raw = h.finish();
    let span = max - MIN_MULTIPLIER;
    let frac = (raw % 10_000) as f64 / 10_000.0;
    let v = MIN_MULTIPLIER + frac * span;
    (v * 100.0).round() / 100.0
}

fn multiplier_for(state: &State, round: u32) -> f64 {
    if state.random {
        random_multiplier(&state.started_players, round, state.multiplier)
    } else {
        state.multiplier
    }
}

fn fmt_mult(m: f64) -> String {
    let rounded = (m * 100.0).round() / 100.0;
    let s = format!("{rounded:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s.is_empty() {
        "0".into()
    } else {
        s
    }
}

pub struct Jog;

impl RoomLogic for Jog {
    type State = State;

    fn fold(state: &mut Self::State, kind: &str, payload: &str) {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !state.players.contains(&pid)
                    && state.players.len() < MAX_PLAYERS
                    && state.phase == Phase::Lobby
                {
                    state.players.push(pid.clone());
                    if state.host.is_none() {
                        state.host = Some(pid);
                    }
                }
            }
            "PLAYER_LEFT" => {
                let pid = payload.trim();
                if let Some(idx) = state.lobby_index(pid) {
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
                let (coins, rounds, random, mult, blind) = parse_start_params(payload);
                state.starting_coins = coins;
                state.total_rounds = rounds;
                state.multiplier = mult;
                state.random = random;
                state.blind = blind;
                state.phase = Phase::Playing;
                state.current_round = 1;
                state.started_players = state.players.clone();
                let n = state.started_players.len();
                state.coins = vec![coins; n];
                state.pending_contrib = vec![None; n];
            }
            "CONTRIBUTE" => {
                // payload: "<pid> <amount>"
                let mut it = payload.splitn(2, ' ');
                let pid = it.next().unwrap_or("");
                let amt = it.next().and_then(|s| s.trim().parse::<i64>().ok());
                if let (Some(idx), Some(a)) = (state.play_index(pid), amt) {
                    if state.phase == Phase::Playing
                        && idx < state.pending_contrib.len()
                        && state.pending_contrib[idx].is_none()
                        && a >= 0
                        && a <= state.coins[idx]
                    {
                        state.pending_contrib[idx] = Some(a);
                        state.coins[idx] -= a;
                    }
                }
            }
            "ROUND_RESULT" => {
                // payload (blind):     "<round> <multiplier> <jar> <payout>"
                // payload (open):      "<round> <multiplier> <jar> <payout> <c0> <c1> ... <cN-1>"
                // Open mode replaces the local coins from the payload
                // tail; blind mode applies payout uniformly to whatever
                // we have.
                let parts: Vec<&str> = payload.split_whitespace().collect();
                let n = state.coins.len();
                if parts.len() == 4 + n {
                    let mut new_coins = vec![0i64; n];
                    let mut ok = true;
                    for i in 0..n {
                        match parts[4 + i].parse::<i64>() {
                            Ok(v) => new_coins[i] = v,
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        state.coins = new_coins;
                    }
                } else if parts.len() == 4 {
                    if let Ok(payout) = parts[3].parse::<i64>() {
                        for c in &mut state.coins {
                            *c += payout;
                        }
                    }
                }
                state.pending_contrib = vec![None; n];
                if state.current_round < state.total_rounds {
                    state.current_round += 1;
                }
            }
            "GAME_END" => {
                // payload: "<c0> <c1> ... <cN-1>" — final reveal.
                let parts: Vec<&str> = payload.split_whitespace().collect();
                let n = state.coins.len();
                if parts.len() == n {
                    let mut new_coins = vec![0i64; n];
                    let mut ok = true;
                    for i in 0..n {
                        match parts[i].parse::<i64>() {
                            Ok(v) => new_coins[i] = v,
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        state.coins = new_coins;
                    }
                }
                state.phase = Phase::Finished;
            }
            "WINNER" | "DRAW" | "CHAT" => {}
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
                if state.players.len() < MIN_PLAYERS_TO_START {
                    return Err(format!("need at least {MIN_PLAYERS_TO_START} players"));
                }
                let (coins, rounds, random, mult, blind) = parse_start_params(payload);
                Ok(vec![(
                    "GAME_STARTED".into(),
                    format!(
                        "{coins} {rounds} {} {} {}",
                        if random { 1 } else { 0 },
                        fmt_mult(mult),
                        if blind { 1 } else { 0 },
                    ),
                )])
            }
            "CONTRIBUTE" => {
                if state.phase != Phase::Playing {
                    return Err("not playing".into());
                }
                let idx = state
                    .play_index(player)
                    .ok_or_else(|| "not in game".to_string())?;
                if idx >= state.pending_contrib.len() {
                    return Err("not in game".into());
                }
                if state.pending_contrib[idx].is_some() {
                    return Err("already contributed this round".into());
                }
                let amt: i64 = payload
                    .trim()
                    .parse()
                    .map_err(|_| "bad amount".to_string())?;
                if amt < 0 {
                    return Err("amount must be non-negative".into());
                }
                if amt > state.coins[idx] {
                    return Err("amount exceeds coins".into());
                }

                let mut out =
                    vec![("CONTRIBUTE".into(), format!("{player} {amt}"))];

                let everyone_in = state
                    .pending_contrib
                    .iter()
                    .enumerate()
                    .all(|(i, c)| i == idx || c.is_some());
                if everyone_in {
                    let n = state.coins.len();
                    let mult = multiplier_for(state, state.current_round);

                    let mut contribs = vec![0i64; n];
                    for i in 0..n {
                        contribs[i] = if i == idx {
                            amt
                        } else {
                            state.pending_contrib[i].unwrap_or(0)
                        };
                    }
                    let jar: i64 = contribs.iter().sum();
                    let pot = ((jar as f64) * mult).floor() as i64;
                    let payout = pot / (n as i64);

                    // Compute post-round balances (server knows them
                    // all). state.coins[i] already excludes contribs[i]
                    // for every i except `idx` (whose CONTRIBUTE is
                    // unfolded). Subtract own amt manually, then add
                    // payout to everyone.
                    let mut finals = vec![0i64; n];
                    for i in 0..n {
                        let base = state.coins[i] - if i == idx { amt } else { 0 };
                        finals[i] = base + payout;
                    }

                    let mut round_payload = format!(
                        "{} {} {} {}",
                        state.current_round,
                        fmt_mult(mult),
                        jar,
                        payout
                    );
                    if !state.blind {
                        for c in &finals {
                            round_payload.push(' ');
                            round_payload.push_str(&c.to_string());
                        }
                    }
                    out.push(("ROUND_RESULT".into(), round_payload));

                    if state.current_round >= state.total_rounds {
                        let mut end_buf = String::new();
                        for (i, c) in finals.iter().enumerate() {
                            if i > 0 {
                                end_buf.push(' ');
                            }
                            end_buf.push_str(&c.to_string());
                        }
                        out.push(("GAME_END".into(), end_buf));

                        let max = *finals.iter().max().unwrap_or(&0);
                        let winners: Vec<usize> = finals
                            .iter()
                            .enumerate()
                            .filter(|(_, c)| **c == max)
                            .map(|(i, _)| i)
                            .collect();
                        if winners.len() == 1 {
                            out.push(("WINNER".into(), winners[0].to_string()));
                        } else {
                            out.push(("DRAW".into(), String::new()));
                        }
                    }
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
        "jar-of-greed"
    }

    fn pending_players(state: &Self::State) -> Vec<String> {
        if state.phase != Phase::Playing {
            return Vec::new();
        }
        state
            .started_players
            .iter()
            .enumerate()
            .filter(|(i, _)| state.pending_contrib.get(*i).copied().flatten().is_none())
            .map(|(_, p)| p.clone())
            .collect()
    }

    fn snapshot(state: &Self::State) -> RoomSnapshot {
        let players = if state.phase == Phase::Lobby {
            state.players.clone()
        } else {
            state.started_players.clone()
        };
        RoomSnapshot {
            phase: phase_str(&state.phase),
            host: state.host.clone(),
            players,
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

    fn drive(s: &mut State, player: &str, kind: &str, payload: &str) {
        let evs = Jog::validate(s, player, kind, payload).expect("validate ok");
        for (k, p) in &evs {
            Jog::fold(s, k, p);
        }
    }

    #[test]
    fn blind_round_result_omits_balances() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        // blind=1
        drive(&mut s, "a", "START", "10 2 0 2 1");
        let evs = Jog::validate(&s, "a", "CONTRIBUTE", "4").unwrap();
        for (k, p) in &evs {
            Jog::fold(&mut s, k, p);
        }
        let evs = Jog::validate(&s, "b", "CONTRIBUTE", "6").unwrap();
        let round_event = evs.iter().find(|(k, _)| k == "ROUND_RESULT").unwrap();
        let parts: Vec<&str> = round_event.1.split_whitespace().collect();
        assert_eq!(parts.len(), 4, "blind ROUND_RESULT hides balances");
        assert_eq!(parts[3], "10");
    }

    #[test]
    fn open_round_result_includes_balances() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        // blind=0
        drive(&mut s, "a", "START", "10 2 0 2 0");
        let evs = Jog::validate(&s, "a", "CONTRIBUTE", "4").unwrap();
        for (k, p) in &evs {
            Jog::fold(&mut s, k, p);
        }
        let evs = Jog::validate(&s, "b", "CONTRIBUTE", "6").unwrap();
        let round_event = evs.iter().find(|(k, _)| k == "ROUND_RESULT").unwrap();
        let parts: Vec<&str> = round_event.1.split_whitespace().collect();
        assert_eq!(parts.len(), 6, "open ROUND_RESULT carries balance tail");
        assert_eq!(parts[4], "16");
        assert_eq!(parts[5], "14");
    }

    #[test]
    fn defaults_to_blind_when_flag_omitted() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        // omit blind flag entirely
        drive(&mut s, "a", "START", "10 1 0 2");
        assert!(s.blind);
    }

    #[test]
    fn fold_round_result_applies_payout_to_everyone() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "c", "JOIN", "");
        drive(&mut s, "a", "START", "20 2 0 3");
        drive(&mut s, "a", "CONTRIBUTE", "10");
        drive(&mut s, "b", "CONTRIBUTE", "5");
        drive(&mut s, "c", "CONTRIBUTE", "0");
        // jar=15, mult=3, pot=45, payout=15
        // a: 20-10+15=25; b: 20-5+15=30; c: 20-0+15=35
        assert_eq!(s.coins, vec![25, 30, 35]);
    }

    #[test]
    fn game_end_reveals_final_balances() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "10 1 0 2");
        drive(&mut s, "a", "CONTRIBUTE", "4");
        let evs = Jog::validate(&s, "b", "CONTRIBUTE", "6").unwrap();
        let end = evs.iter().find(|(k, _)| k == "GAME_END").unwrap();
        let parts: Vec<&str> = end.1.split_whitespace().collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "16");
        assert_eq!(parts[1], "14");
    }

    #[test]
    fn fractional_multiplier_floors_for_three_players() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "c", "JOIN", "");
        drive(&mut s, "a", "START", "20 1 0 1.5");
        drive(&mut s, "a", "CONTRIBUTE", "10");
        drive(&mut s, "b", "CONTRIBUTE", "5");
        drive(&mut s, "c", "CONTRIBUTE", "0");
        // jar=15, mult=1.5, pot=floor(22.5)=22, payout=22/3=7
        // a: 20-10+7=17; b: 20-5+7=22; c: 20-0+7=27
        assert_eq!(s.coins, vec![17, 22, 27]);
    }

    #[test]
    fn cannot_start_with_one_player() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        assert!(Jog::validate(&s, "a", "START", "10 1 0 2").is_err());
    }

    #[test]
    fn cannot_contribute_more_than_coins() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "5 3 0 2");
        assert!(Jog::validate(&s, "a", "CONTRIBUTE", "6").is_err());
    }

    #[test]
    fn cannot_contribute_twice_per_round() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "c", "JOIN", "");
        drive(&mut s, "a", "START", "10 3 0 2");
        drive(&mut s, "a", "CONTRIBUTE", "1");
        assert!(Jog::validate(&s, "a", "CONTRIBUTE", "1").is_err());
    }

    #[test]
    fn three_way_tie_emits_draw() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "c", "JOIN", "");
        drive(&mut s, "a", "START", "10 1 0 2");
        drive(&mut s, "a", "CONTRIBUTE", "3");
        drive(&mut s, "b", "CONTRIBUTE", "3");
        drive(&mut s, "c", "CONTRIBUTE", "3");
        assert_eq!(s.coins, vec![13, 13, 13]);
        assert_eq!(s.phase, Phase::Finished);
    }

    #[test]
    fn random_multiplier_in_range_and_deterministic() {
        let players = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let m1 = random_multiplier(&players, 1, 3.0);
        let m2 = random_multiplier(&players, 1, 3.0);
        assert_eq!(m1, m2);
        assert!(m1 >= MIN_MULTIPLIER && m1 <= 3.0);
    }

    #[test]
    fn replay_reconstructs_three_players_blind() {
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("PLAYER_JOINED", "c"),
            ("GAME_STARTED", "10 2 0 2 1"),
            ("CONTRIBUTE", "a 4"),
            ("CONTRIBUTE", "b 6"),
            ("CONTRIBUTE", "c 0"),
            // jar=10, pot=20, payout=20/3=6
            ("ROUND_RESULT", "1 2 10 6"),
            ("CONTRIBUTE", "a 0"),
            ("CONTRIBUTE", "b 10"),
            ("CONTRIBUTE", "c 0"),
            ("ROUND_RESULT", "2 2 10 6"),
            ("GAME_END", "18 6 22"),
            ("WINNER", "2"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            Jog::fold(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.coins, vec![18, 6, 22]);
    }

    #[test]
    fn replay_reconstructs_three_players_open() {
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("PLAYER_JOINED", "c"),
            ("GAME_STARTED", "10 2 0 2 0"),
            ("CONTRIBUTE", "a 4"),
            ("CONTRIBUTE", "b 6"),
            ("CONTRIBUTE", "c 0"),
            ("ROUND_RESULT", "1 2 10 6 12 10 16"),
            ("CONTRIBUTE", "a 0"),
            ("CONTRIBUTE", "b 10"),
            ("CONTRIBUTE", "c 0"),
            ("ROUND_RESULT", "2 2 10 6 18 6 22"),
            ("GAME_END", "18 6 22"),
            ("WINNER", "2"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            Jog::fold(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.coins, vec![18, 6, 22]);
    }
}
