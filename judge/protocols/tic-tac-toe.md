# Tic Tac Toe Protocol

## Game Overview
- 2 players (X and O)
- 3x3 grid
- First to get 3 in a row wins

## Message Flow

### 1. Game Start
**Server → Player 1:**
```
START X
```

**Server → Player 2:**
```
START O
```

### 2. Turn Information (broadcast to all)
**Server → All Players:**
```
TURN {turn_number}
```

### 3. Turn Notification (current player only)
**Server → Current Player:**
```
YOUR_TURN
```

### 4. Move Input
**Player → Server:**
```
MOVE {row} {col}
```

Example:
```
MOVE 1 2
```

### 5. Board State (after each move)
**Server → All Players:**
```
BOARD X.O.X.O..
```

The board is sent as a single-line message with 9 characters representing the 3x3 grid (top-left to bottom-right).

### 6. Game End
**Server → Player:**
```
SCORE {your_final_score}
```

### 7. Graceful Exit
**Server → Player:**
```
END
```

## Rules
- Grid positions are 0-indexed (0, 1, 2)
- Player X always goes first
- Players alternate turns
- Win conditions: 3 in a row (horizontal, vertical, or diagonal)
- Draw if board is full with no winner

## Board Representation
- `X` = Player 1's mark
- `O` = Player 2's mark
- `.` = Empty space
- 9 characters in grid order: top-left to bottom-right (row 0 col 0, row 0 col 1, ..., row 2 col 2)
- Single line format in BOARD messages

## Reconnection

When a player reconnects during an active game, the server sends the current game state:

```
START {X|O}
BOARD X.O.X.O..
TURN {current_turn_number}
YOUR_TURN
```

Or if the game is finished:

```
START {X|O}
BOARD X.O.X.O.O
SCORE {your_final_score}
END
```

The reconnecting player receives their assigned symbol (X or O) and the current board state, allowing them to resume play.

## Error Conditions
- Invalid format (not "MOVE row col") → `WrongAnswer`
- Invalid coordinates (not 0-2) → `WrongAnswer`
- Position already occupied → `WrongAnswer`
- Timeout → `TimeLimitExceeded`
