# Prisoners Dilemma Protocol

## Game Overview
- 2 players
- Random number of rounds (7-13, configurable)
- Highest total score wins

## Message Flow

### 1. Game Start
**Server → Player:**
```
START
```

### 2. Move Input (per round)
**Player → Server:**
```
C
```
or
```
COOPERATE
```
or
```
D
```
or
```
DEFECT
```

### 3. Round Result
**Server → Player:**
```
RESULT {opponent_move} {your_move} {opponent_score} {your_score}
```

Example:
```
RESULT D C 20 15
```

### 4. Game End
**Server → Player:**
```
SCORE {your_final_score}
```

### 5. Graceful Exit
**Server → Player:**
```
END
```

## Scoring Matrix
| Your Move | Opponent Move | Your Points | Opponent Points |
|-----------|---------------|-------------|-----------------|
| C         | C             | 3           | 3               |
| C         | D             | 0           | 5               |
| D         | C             | 5           | 0               |
| D         | D             | 1           | 1               |

## Rules
- C = Cooperate, D = Defect
- Both forms (C/COOPERATE, D/DEFECT) are accepted
- Player with highest total score after all rounds wins
- Number of rounds is random and unknown to players

## Error Conditions
- Invalid move (not C/COOPERATE/D/DEFECT) → `WrongAnswer`
- Timeout → `TimeLimitExceeded`
