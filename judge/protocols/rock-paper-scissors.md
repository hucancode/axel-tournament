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

## Error Conditions
- Invalid move (not ROCK/PAPER/SCISSORS) → `WrongAnswer`
- Timeout → `TimeLimitExceeded`
