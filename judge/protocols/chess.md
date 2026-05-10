# Chess

`game_id = chess`. Two players on an 8×8 board. Player 0 = white (moves
first), player 1 = black. Standard FIDE rules: piece movement, castling,
en passant, promotion, check / checkmate / stalemate, fifty-move rule,
threefold repetition, and insufficient-material draws.

Squares are written in algebraic form (`a1`..`h8`). Files run `a..h`
(white's queenside to white's kingside); ranks run `1..8` (white's back
rank to black's back rank).

## Actions (client → server `ACT`)

| kind    | payload                  | when valid                                     |
|---------|--------------------------|------------------------------------------------|
| `JOIN`  | (none)                   | lobby phase, room not full                     |
| `LEAVE` | (none)                   | always (no-op if not in room)                  |
| `START` | (none)                   | host, lobby phase, exactly 2 players           |
| `MOVE`  | `<from> <to> [promo]`    | playing phase, your turn, move is fully legal  |
| `CHAT`  | `<msg>`                  | always                                         |

`<from>` and `<to>` are algebraic squares. `[promo]` is `q|r|b|n` and is
required when (and only when) the move advances a pawn to its promotion
rank. It is ignored otherwise.

A move is legal iff:

- It belongs to the side to move and follows the piece's movement rules.
- It does not leave the mover's king in check.
- For castling: king and the corresponding rook have not moved, no
  pieces stand between them, the king is not currently in check, and
  the king does not pass through (or land on) a square attacked by the
  opponent. Castling is encoded as the king's two-square move
  (`e1 g1`, `e1 c1`, `e8 g8`, `e8 c8`).
- For en passant: the previous move was a pawn double-step into the
  capturing pawn's adjacent file.

## Events (server → client `EVENT`)

| kind            | payload                       | meaning                              |
|-----------------|-------------------------------|--------------------------------------|
| `PLAYER_JOINED` | `<pid>`                       | player added to room                 |
| `PLAYER_LEFT`   | `<pid>`                       | player removed                       |
| `HOST_CHANGED`  | `<pid>`                       | host transferred                     |
| `GAME_STARTED`  | (empty)                       | match started; player 0 = white      |
| `MOVE`          | `<pid> <from> <to> <flag>`    | move applied (see flags below)       |
| `CHECK`         | `<player_idx>`                | mover put `<player_idx>` in check    |
| `WINNER`        | `<player_idx>`                | terminal: `<player_idx>` won         |
| `DRAW`          | `<reason>`                    | terminal draw (see reasons below)    |
| `CHAT`          | `<pid> <msg>`                 | chat (no state mutation)             |

### MOVE flag

Single token describing how to render / apply the move beyond the
plain `from → to` swap.

| flag        | meaning                                                |
|-------------|--------------------------------------------------------|
| `-`         | ordinary move (including pawn double-step and capture) |
| `K`         | kingside castle (rook moves `h→f`)                     |
| `Q`         | queenside castle (rook moves `a→d`)                    |
| `E`         | en passant (captured pawn sits on `to`'s file, mover's rank) |
| `q|r|b|n`   | promotion to queen / rook / bishop / knight            |

Captures are not flagged separately; the destination square implies
capture from the receiver's board state. `CHECK` is emitted only when
the move puts the opponent in check **without** also delivering
checkmate; checkmate emits `WINNER` instead.

### DRAW reason

| reason         | meaning                                        |
|----------------|------------------------------------------------|
| `STALEMATE`    | side to move has no legal move and is not in check |
| `FIFTY_MOVE`   | 50 full moves with no pawn move and no capture |
| `THREEFOLD`    | the same position has occurred three times     |
| `INSUFFICIENT` | neither side has mating material               |

## Reconnect

See [`wire.md`](wire.md). Replay the event stream, applying each `MOVE`
flag to keep castling rights, en passant target, and clocks in sync.
