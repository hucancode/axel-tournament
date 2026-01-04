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

## Reconnection

When a player reconnects during an active game, the server sends the complete game history with results in player perspective (opponent_move, your_move, opponent_score, your_score):

```
START
RESULT D C 5 0
RESULT C D 10 0
RESULT D D 11 1
RESULT {opponent_move} {your_move} {opponent_cumulative_score} {your_cumulative_score}
```

This replays all completed rounds with cumulative scores, matching the live gameplay format. The reconnecting player can then continue from the current round.

If the game is finished:

```
START
RESULT D C 5 0
...
RESULT {opponent_move} {your_move} {final_opponent_score} {final_your_score}
SCORE {your_final_score}
END
```

## Error Conditions
- Invalid move (not C/COOPERATE/D/DEFECT) → `WrongAnswer`
- Timeout → `TimeLimitExceeded`
