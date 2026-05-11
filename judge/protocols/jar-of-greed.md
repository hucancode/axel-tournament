# Jar of Greed

`game_id = jar-of-greed`. 2..=8 players. Each starts with a configurable
coin balance. Every round every player secretly contributes some of
their coins to a shared jar; the jar is then multiplied by a
host-configured factor strictly greater than 1 (fixed value, or
randomized per round) and split evenly back to every player. After the
configured number of rounds, final coin balances are the score —
highest stack wins.

All coin arithmetic floors the result. Odd-coin payout remainders are
discarded.

## Actions (client → server `ACT`)

| kind         | payload                                  | when valid                                                |
|--------------|------------------------------------------|-----------------------------------------------------------|
| `JOIN`       | (none)                                   | lobby phase, room not full                                |
| `LEAVE`      | (none)                                   | always (no-op if not in room)                             |
| `START`      | `[<coins> <rounds> <random> <mult>]`     | host, lobby phase, at least 2 players                     |
| `CONTRIBUTE` | `<amount>`                               | playing phase, hasn't contributed this round, `0 <= amount <= your_coins` |
| `CHAT`       | `<msg>`                                  | always                                                    |

START defaults when omitted: `10 5 0 2 1` (10 starting coins, 5 rounds,
fixed mode, multiplier ×2, blind balances). All five fields are
positional and trailing fields take their defaults if absent.

- `<coins>`: starting coin balance per player. Clamped to `1..=1_000_000`.
- `<rounds>`: total rounds played. Clamped to `1..=1000`.
- `<random>`: `0` for fixed multiplier, `1` for per-round randomization.
- `<mult>`: floating-point multiplier. When `random == 0`, used as the
  fixed factor for every round. When `random == 1`, used as the upper
  bound of the per-round draw (lower bound is `1.01`). Clamped to
  `1.01..=10.0`; values at or below 1 are coerced up to `1.01`.
- `<blind>`: `0` reveals every player's balance after each round's
  payout (in `ROUND_RESULT`); `1` (the default) hides per-player
  balances until `GAME_END`.

## Events (server → client `EVENT`)

| kind            | payload                                                            | meaning                                       |
|-----------------|--------------------------------------------------------------------|-----------------------------------------------|
| `PLAYER_JOINED` | `<pid>`                                                            | player added                                  |
| `PLAYER_LEFT`   | `<pid>`                                                            | player removed                                |
| `HOST_CHANGED`  | `<pid>`                                                            | host transferred                              |
| `GAME_STARTED`  | `<coins> <rounds> <random> <mult> <blind>`                         | match parameters fixed; player roster frozen  |
| `CONTRIBUTE`    | `<pid> <amount>`                                                   | player committed an amount this round         |
| `ROUND_RESULT`  | `<round> <multiplier> <jar> <payout>` (blind), or `<round> <multiplier> <jar> <payout> <c0> ... <c_{N-1}>` (open) | round resolved; balance tail present iff `blind == 0` |
| `GAME_END`      | `<c0> <c1> ... <c_{N-1}>`                                          | match over; final balances                    |
| `WINNER`        | `<player_idx>`                                                     | unique-max stack at GAME_END                  |
| `DRAW`          | (empty)                                                            | tied max at GAME_END                          |
| `CHAT`          | `<pid> <msg>`                                                      | chat (no state mutation)                      |

Player indices are assigned by join order; index 0 is the first joiner.
The roster is frozen at GAME_STARTED — the per-player coin and
contribution arrays are sized once and never resized. A LEAVE during
play removes the player from the lobby list but cannot resize those
arrays; the round simply never resolves and the per-turn timeout
watcher terminates the match as a forfeit.

`N` in the `ROUND_RESULT` / `GAME_END` payloads is the number of
players at the moment GAME_STARTED was emitted.

The `<multiplier>` field in `ROUND_RESULT` is the actual factor applied
that round (matters for `random == 1`). It is written with at most two
decimal places; trailing zeros are trimmed.

## Hidden information

Two layers of secrecy:

1. **Contribution amounts during the contributing phase.** CONTRIBUTE
   events carry `<pid> <amount>` in plain text — required because the
   event log is the single source of truth for state and every
   state-changing field has to be replayable. The "you don't know what
   others put in *this round* until the round resolves" property is
   therefore enforced at the human web client: render only your own
   amount in real time, never opponents'. This matches the existing
   poker hole-card pattern.

2. **Per-player balances during play.** Controlled by the `blind`
   START flag:
   - `blind = 1` (default): `ROUND_RESULT` payloads omit per-player
     balances. Each player can still derive their own balance from
     their own CONTRIBUTE events plus the per-round payout. Other
     players' balances are revealed only by `GAME_END`.
   - `blind = 0`: every `ROUND_RESULT` carries the full per-player
     balance tail, so spectators and replay viewers see each player's
     stack after every payout.

For bot tournaments where every player is a subprocess and the log is
the audit trail, both layers are visible to all participants regardless
of `blind`. The flag exists for human-mode UX, not as a cryptographic
guarantee.

## Round resolution

Given pre-round balances `c[0..N]` and contributions `a[0..N]` with
multiplier `m`:

```
jar    = sum(a[i])
pot    = floor(jar * m)
payout = pot / N                       -- integer division, remainder discarded
c[i]'  = c[i] - a[i] + payout          -- for each i in 0..N
```

`payout` is identical for every player, so the strategic asymmetry
comes entirely from the contribution split, not from the payout split.

## Lifecycle

```
lobby      -- waiting for >=2 players + START
playing
  round 1 -> wait for every CONTRIBUTE -> ROUND_RESULT
  round 2 -> ...
  ...
finished   -- ROUND_RESULT for final round triggers GAME_END + WINNER/DRAW
```

## Reconnect

See [`wire.md`](wire.md). Replay events; the multiplier travels in
`ROUND_RESULT` so replay does not need an RNG seed even in random mode.
