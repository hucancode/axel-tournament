# Rock Paper Scissors

`game_id = rock-paper-scissors`. Two players. Best score after a fixed
number of rounds (default 5) wins.

## Actions (client → server `ACT`)

| kind    | payload                       | when valid                        |
|---------|-------------------------------|-----------------------------------|
| `JOIN`  | (none)                        | lobby phase, room not full        |
| `LEAVE` | (none)                        | always (no-op if not in room)     |
| `START` | (none)                        | host, lobby phase, ≥ 2 players    |
| `MOVE`  | `ROCK \| PAPER \| SCISSORS`   | playing phase, hasn't moved this round |
| `CHAT`  | `<msg>`                       | always                            |

`MOVE` accepts the short forms `R`, `P`, `S` as aliases.

## Events (server → client `EVENT`)

| kind            | payload                              | meaning                          |
|-----------------|--------------------------------------|----------------------------------|
| `PLAYER_JOINED` | `<pid>`                              | player added to room             |
| `PLAYER_LEFT`   | `<pid>`                              | player removed                   |
| `HOST_CHANGED`  | `<pid>`                              | host transferred                 |
| `GAME_STARTED`  | `<total_rounds>`                     | match started                    |
| `MOVE`          | `<pid> <ROCK\|PAPER\|SCISSORS>`      | player committed a move          |
| `ROUND_RESULT`  | `<round> <m0> <m1> <s0> <s1>`        | round resolved                   |
| `GAME_END`      | `<s0> <s1>`                          | final cumulative scores          |
| `CHAT`          | `<pid> <msg>`                        | chat (no state mutation)         |

`m0`/`m1` are the player-0 and player-1 moves for the round; `s0`/`s1`
are the cumulative scores after the round (also used for `GAME_END`).
Player indices come from join order; the first player is index 0.

## Scoring

Rock beats Scissors. Paper beats Rock. Scissors beats Paper. Equal moves
are a draw and award no points. The round winner gains 1 point.

## Reconnect

Reconnect is a wire-level concern, not a per-game concern. See
[`wire.md`](wire.md). The client replays the events above and renders
state directly; there is no separate reconnect format.
