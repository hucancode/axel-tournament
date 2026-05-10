# Poker

`game_id = poker`. Two players. Heads-up no-limit Texas Hold'em.

The judge plays out one or more hands. The dealer button alternates each
hand. After all hands (or as soon as either player busts) the room ends
and per-player chip stacks are reported.

## Actions (client → server `ACT`)

| kind     | payload                                         | when valid                                                |
|----------|-------------------------------------------------|-----------------------------------------------------------|
| `JOIN`   | (none)                                          | lobby phase, room not full                                |
| `LEAVE`  | (none)                                          | always (no-op if not in room)                             |
| `START`  | `[<num_hands> <starting_stack> <small_blind>]`  | host, lobby phase, exactly 2 players                      |
| `CHAT`   | `<msg>`                                         | always                                                    |
| `FOLD`   | (none)                                          | playing phase, your turn, there is a bet to call          |
| `CHECK`  | (none)                                          | playing phase, your turn, no outstanding bet              |
| `CALL`   | (none)                                          | playing phase, your turn, opponent has bet                |
| `BET`    | `<amount>`                                      | playing phase, your turn, current bet is 0                |
| `RAISE`  | `<to-amount>`                                   | playing phase, your turn, raise meets minimum             |
| `ALLIN`  | (none)                                          | playing phase, your turn, stack > 0                       |

Defaults when `START` payload is omitted: `10 1000 10` (10 hands, 1000
chip starting stack, 10 small blind / 20 big blind).

`<to-amount>` for `RAISE` is the *total* this-street contribution after
the raise — not the raise increment. A min-raise is therefore
`current_bet + max(big_blind, last_raise_size)`.

## Events (server → client `EVENT`)

| kind            | payload                                              | meaning                                            |
|-----------------|------------------------------------------------------|----------------------------------------------------|
| `PLAYER_JOINED` | `<pid>`                                              | player added                                       |
| `PLAYER_LEFT`   | `<pid>`                                              | player removed                                     |
| `HOST_CHANGED`  | `<pid>`                                              | host transferred                                   |
| `GAME_STARTED`  | `<num_hands> <starting_stack> <small_blind>`         | match parameters fixed                             |
| `HAND_STARTED`  | `<hand_no> <dealer_idx> <hole0> <hole1>`             | start of a hand; concrete hole cards baked in      |
| `STREET`        | `<flop\|turn\|river> <cards...>`                     | community cards revealed (flop=3, turn/river=1)    |
| `ACTION`        | `<pid> <FOLD\|CHECK\|CALL\|BET\|RAISE\|ALLIN> [amt]` | a player's chosen action this street               |
| `POT`           | `<amount>`                                           | committed pot after the most recent action / street|
| `HAND_END`      | `<winner_idx\|-1> <chips0> <chips1> <reason>`        | hand resolved (`reason` ∈ {`FOLD`,`SHOWDOWN`,`SPLIT`}) |
| `GAME_END`      | `<chips0> <chips1>`                                  | match over; final stacks                           |
| `WINNER`        | `<player_idx>`                                       | back-compat shim: who has more chips at GAME_END   |
| `DRAW`          | (empty)                                              | back-compat shim: equal chips at GAME_END          |
| `CHAT`          | `<pid> <msg>`                                        | chat (no state mutation)                           |

Player indices are assigned by join order; index 0 is the first joiner.
`hole0` and `hole1` are the two-character cards belonging to player 0
and player 1 respectively (e.g. `Ah Td 2c 7s`).

## Card encoding

Two ASCII characters, rank then suit.

- Ranks: `2 3 4 5 6 7 8 9 T J Q K A`.
- Suits: `c` (clubs), `d` (diamonds), `h` (hearts), `s` (spades).

Examples: `As`, `Td`, `2c`. The deck is a 52-card standard deck.

## Hole card visibility

Hole cards appear in `HAND_STARTED` payloads in plain text and are
therefore visible to every connected client. This is a deliberate
trade-off: the protocol is event-sourced and replayable, so any state
reachable from the log must travel over the same channel for every
observer. There is no per-recipient redaction in the wire layer.

For human play this means the "secret hole card" abstraction has to be
imposed by clients (only render your own card face-up). For bot tournaments
where both players are subprocesses and the log is the audit trail, this
is exactly what the runtime needs.

## Heads-up rules

- Dealer button alternates every hand. Hand 1 dealer is index 0.
- Dealer posts the small blind. Opponent posts the big blind (= 2 × SB).
- Pre-flop: dealer (small blind) acts first.
- Flop, turn, river: opponent (big blind position) acts first.
- A player whose stack falls below a posted blind goes all-in for what
  they have. The deficit does not create side-pot complexity in the
  heads-up case — the pot simply caps to twice the smaller stack at
  showdown and the loser goes broke.

## Betting rules

- `CHECK` is legal only when the current bet to call is 0.
- `CALL` matches the opponent's contribution this street; if the player
  cannot cover, it is treated as `ALLIN`.
- `BET` is legal only when the current bet is 0. Amount must be at
  least the big blind and at most the player's stack.
- `RAISE <to>` must satisfy `to >= current_bet + max(big_blind, last_raise_size)`
  and `to <= player_stack + already_in_this_street`. ALLIN below this
  threshold is permitted but does not reopen the action for an opponent
  who has already acted this round.
- `ALLIN` puts the player's whole remaining stack in. Treated as a
  raise if it exceeds the current bet, otherwise as a call.
- A street ends when both players have acted at least once *and* both
  contributions are equal, or one player is all-in and the other has
  matched or folded.

## Hand evaluation

Standard 7-card best-five ranking. From strongest to weakest:

1. Straight flush
2. Four of a kind
3. Full house
4. Flush
5. Straight (wheel `A-2-3-4-5` supported)
6. Three of a kind
7. Two pair
8. One pair
9. High card

Ties are broken by kickers in normal poker order. Tied seven-card hands
result in a split pot (`HAND_END ... SPLIT`).

## Lifecycle

```
lobby                      -- waiting for 2 players + START
playing
  hand 1 preflop -> flop -> turn -> river -> showdown / fold
  hand 2 ...
  ...
finished                   -- last hand done OR a player busted
```

After `GAME_END`, a single `WINNER <idx>` (or `DRAW`) event is appended
so the match-runner's existing terminal-event parser, which extracts
per-player scalar scores, still keys correctly during the chip-scoring
roll-out. The chip totals in `GAME_END` are the source of truth for
ELO-equivalent accumulation.

## Reconnect

See [`wire.md`](wire.md). Replay events; both `HAND_STARTED` and
`STREET` carry the concrete cards, so replay does not need a deck or
seed.
