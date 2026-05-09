# Prisoners Dilemma

`game_id = prisoners-dilemma`. Two players. Iterated PD with a fixed round
count (default 10). Highest cumulative score wins.

## Actions (client → server `ACT`)

| kind    | payload         | when valid                                |
|---------|-----------------|-------------------------------------------|
| `JOIN`  | (none)          | lobby phase, room not full                |
| `LEAVE` | (none)          | always (no-op if not in room)             |
| `START` | (none)          | host, lobby phase, exactly 2 players      |
| `MOVE`  | `C \| D`        | playing phase, hasn't moved this round    |
| `CHAT`  | `<msg>`         | always                                    |

`MOVE` accepts the long forms `COOPERATE` and `DEFECT` as aliases.

## Events (server → client `EVENT`)

| kind            | payload                        | meaning                            |
|-----------------|--------------------------------|------------------------------------|
| `PLAYER_JOINED` | `<pid>`                        | player added to room               |
| `PLAYER_LEFT`   | `<pid>`                        | player removed                     |
| `HOST_CHANGED`  | `<pid>`                        | host transferred                   |
| `GAME_STARTED`  | `<total_rounds>`               | match started                      |
| `MOVE`          | `<pid> <C\|D>`                 | move committed                     |
| `ROUND_RESULT`  | `<round> <m0> <m1> <s0> <s1>`  | round resolved (cumulative scores) |
| `GAME_END`      | `<s0> <s1>`                    | final cumulative scores            |
| `CHAT`          | `<pid> <msg>`                  | chat (no state mutation)           |

Player indices are assigned by join order; index 0 is the first joiner.

## Payoff matrix

| `m0` | `m1` | Δscore for player 0 | Δscore for player 1 |
|------|------|---------------------|---------------------|
| C    | C    | 3                   | 3                   |
| C    | D    | 0                   | 5                   |
| D    | C    | 5                   | 0                   |
| D    | D    | 1                   | 1                   |

## Reconnect

See [`wire.md`](wire.md). Replay events and render.
