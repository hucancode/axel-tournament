# Rock Paper Scissors Protocol

## Game Overview
- 2 players
- Random number of rounds (3-7, configurable)
- Best score wins

## Message Flow

### 1. Game Start
**Server → Player:**
```
START
```

### 2. Move Input (per round)
**Player → Server:**
```
ROCK
```
or
```
PAPER
```
or
```
SCISSORS
```

### 3. Round Result
**Server → Player:**
```
ROUND {round_number} SCORE {player_score} {opponent_score}
```

Example:
```
ROUND 1 SCORE 1 0
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

## Rules
- Rock beats Scissors
- Paper beats Rock  
- Scissors beats Paper
- Same moves result in a draw (no points)
- Winner gets 1 point per round
- Player with most points after all rounds wins
- Number of rounds is random and unknown to players

## Reconnection

When a player reconnects during an active game, the server sends:

```
START
ROUND 1 SCORE 0 0
ROUND 2 SCORE 1 0
ROUND 3 SCORE 1 1
ROUND {current_round} SCORE {current_p1_score} {current_p2_score}
```

This replays all completed rounds with cumulative scores, matching the live gameplay format. The reconnecting player can then continue from the current round.

If the game is finished:

```
START
ROUND 1 SCORE 0 0
...
ROUND {total_rounds} SCORE {final_p1_score} {final_p2_score}
SCORE {player_final_score}
END
```

## Error Conditions
- Invalid move (not ROCK/PAPER/SCISSORS) → `WrongAnswer`
- Timeout → `TimeLimitExceeded`
