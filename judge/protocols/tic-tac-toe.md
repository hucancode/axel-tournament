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

### 2. Turn Notification
**Server → Current Player:**
```
YOUR_TURN
```

### 3. Move Input
**Player → Server:**
```
MOVE {row} {col}
```

Example:
```
MOVE 1 2
```

### 4. Board State (after each move)
**Server → All Players:**
```
X.O
.X.
O..
```

### 5. Game End
**Server → Player:**
```
SCORE {your_final_score}
```

### 6. Graceful Exit
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
- Each row on a new line

## Error Conditions
- Invalid format (not "MOVE row col") → `WrongAnswer`
- Invalid coordinates (not 0-2) → `WrongAnswer`
- Position already occupied → `WrongAnswer`
- Timeout → `TimeLimitExceeded`
