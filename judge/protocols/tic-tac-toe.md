# Tic Tac Toe

`game_id = tic-tac-toe`. Two players on a 3×3 grid. First to align three
marks (row, column, or diagonal) wins.

## Actions (client → server `ACT`)

| kind    | payload          | when valid                                       |
|---------|------------------|--------------------------------------------------|
| `JOIN`  | (none)           | lobby phase, room not full                       |
| `LEAVE` | (none)           | always (no-op if not in room)                    |
| `START` | (none)           | host, lobby phase, exactly 2 players             |
| `MOVE`  | `<row> <col>`    | playing phase, your turn, cell empty, 0 ≤ r,c ≤ 2|
| `CHAT`  | `<msg>`          | always                                           |

## Events (server → client `EVENT`)

| kind            | payload                  | meaning                         |
|-----------------|--------------------------|---------------------------------|
| `PLAYER_JOINED` | `<pid>`                  | player added to room            |
| `PLAYER_LEFT`   | `<pid>`                  | player removed                  |
| `HOST_CHANGED`  | `<pid>`                  | host transferred                |
| `GAME_STARTED`  | (empty)                  | match started; player 0 = X     |
| `MOVE`          | `<pid> <row> <col>`      | mark placed                     |
| `WINNER`        | `<player_idx>`           | terminal: 0 or 1 won            |
| `DRAW`          | (empty)                  | terminal: full board, no winner |
| `CHAT`          | `<pid> <msg>`            | chat (no state mutation)        |

Player indices are assigned by join order. Player 0 plays X and moves
first. Whose turn it is can be derived from the count and indices of the
`MOVE` events seen so far.

## Reconnect

See [`wire.md`](wire.md). Replay events and render.
