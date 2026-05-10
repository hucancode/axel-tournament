# Xiangqi (Chinese Chess)

`game_id = xiangqi`. Two players on a 9-file by 10-rank board.

- Player 0 = **red** (帥/Shuai). Red occupies rows 0–4 at the start and
  moves first.
- Player 1 = **black** (將/Jiang). Black occupies rows 5–9 at the start.

Coordinates are written `<file><rank>` where file is `a..i` (column
0..8, left to right from red's perspective) and rank is `0..9` (row 0
is red's back row, row 9 is black's back row). Example: `e0` is red's
general starting square; `e9` is black's general starting square.

## Pieces

Letter = piece kind. Case = side. Uppercase is red, lowercase is black.

| letter | name (red / black) | movement summary                                      |
|--------|--------------------|-------------------------------------------------------|
| K / k  | General 帥 / 將    | 1 step orthogonal, must stay in own palace            |
| A / a  | Advisor 仕 / 士    | 1 step diagonal, must stay in own palace              |
| E / e  | Elephant 相 / 象   | exactly 2 steps diagonal, blocked at midpoint, no river |
| H / h  | Horse 馬           | knight L; blocked if the orthogonal "leg" is occupied |
| R / r  | Rook 車            | any orthogonal distance, no jumping                   |
| C / c  | Cannon 炮          | moves like rook; captures by jumping exactly one piece|
| P / p  | Pawn 兵 / 卒       | forward 1; after crossing river, also sideways 1      |

Palace: red palace is files c..e × ranks 0..2 (cols 3..5, rows 0..2);
black palace is files c..e × ranks 7..9 (cols 3..5, rows 7..9).
River: between rank 4 and rank 5. Elephants may not cross. Pawns gain
sideways movement once they have crossed.

**Flying General**: the two generals may not face each other along an
otherwise empty file. A move that would result in such a configuration
is illegal.

A move that would leave the mover's own general in check is illegal.

## Actions (client → server `ACT`)

| kind    | payload          | when valid                                           |
|---------|------------------|------------------------------------------------------|
| `JOIN`  | (none)           | lobby phase, room not full                           |
| `LEAVE` | (none)           | always (no-op if not in room)                        |
| `START` | (none)           | host, lobby phase, exactly 2 players                 |
| `MOVE`  | `<from> <to>`    | playing phase, your turn, legal piece move           |
| `CHAT`  | `<msg>`          | always                                               |

`<from>` and `<to>` are `<file><rank>` strings, e.g. `MOVE e0 e1` to
push the red general one rank forward.

## Events (server → client `EVENT`)

| kind            | payload                  | meaning                              |
|-----------------|--------------------------|--------------------------------------|
| `PLAYER_JOINED` | `<pid>`                  | player added to room                 |
| `PLAYER_LEFT`   | `<pid>`                  | player removed                       |
| `HOST_CHANGED`  | `<pid>`                  | host transferred                     |
| `GAME_STARTED`  | (empty)                  | board initialised; player 0 = red    |
| `MOVE`          | `<pid> <from> <to>`      | one half-move applied                |
| `WINNER`        | `<player_idx>`           | terminal: 0 or 1 won                 |
| `CHAT`          | `<pid> <msg>`            | chat (no state mutation)             |

Player indices are assigned by join order. Player 0 is red and moves
first. Whose turn it is can be derived from the count of `MOVE` events
since `GAME_STARTED`.

## Termination

Xiangqi rooms always end with a `WINNER`. There is no `DRAW` event.

- **General captured**: the side that captured the opposing general
  wins. The capturing `MOVE` event is followed immediately by
  `WINNER`.
- **Stalemate**: a side that has no legal move on its turn loses
  (xiangqi treats stalemate as a loss for the stuck side). The server
  emits `WINNER <other_side>` without a preceding `MOVE`.

## Reconnect

See [`wire.md`](wire.md). Replay events and rebuild the board from
the move list.
