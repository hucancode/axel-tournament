# Judge Server

A single, scalable judge server that executes and validates games (Tic-Tac-Toe, Rock-Paper-Scissors, Prisoner's Dilemma) with both automated (Docker) and interactive (WebSocket) player implementations.

## Architecture

### Core Design Principles

1. **Game Logic Independence**: Game logic is completely decoupled from player implementation
   - Games only accept inputs and return results
   - Same game code works for both Docker-based bots and human WebSocket players

2. **Horizontal Scalability**: Multiple game server instances can run in parallel
   - Each instance independently claims rooms and matches from the database
   - No coordination needed between servers

3. **Capacity-Based Claiming**: Fair load distribution across servers
   - Idle servers (0% load) claim immediately (0ms delay)
   - Busy servers (90% load) wait 1 second before claiming
   - At capacity servers don't claim new work

4. **Atomic Database Operations**: Race-free claiming via status updates
   - To claim a match: `UPDATE match SET status='queued' WHERE status='pending'`
   - If update returns empty, another server already claimed it

### Game Flow

#### Automated Matches (Tournament Bots)

```
1. Tournament creates match with status='pending'
   ↓
2. Match watcher detects pending match
   ↓
3. Server calculates claim delay based on load
   ↓
4. Atomic claim: UPDATE status='queued' WHERE status='pending'
   ↓
5. If successful → create DockerPlayers, execute game
   ↓
6. Game executor runs loop:
   - Send state to both players (stdin)
   - Receive moves with timeout (stdout)
   - Apply moves to game state
   - Repeat until game over
   ↓
7. Update match with scores, status='completed'
```

#### Interactive Matches (Human Players)

```
1. User creates room (status='waiting')
   ↓
2. Players join room via API
   ↓
3. Host starts game → API sets room status='playing'
   ↓
4. Players connect to WebSocket /room/:room_id
   ↓
5. When all players connected → start game execution
   ↓
6. Game executor runs loop:
   - Send state via WebSocket
   - Receive moves from WebSocket messages
   - Apply moves to game state
   - Broadcast updates
   - Repeat until game over
   ↓
7. Update room status='finished', save scores
```

## Configuration

Environment variables:

```bash
SERVER_PORT=8080                    # HTTP/WebSocket port
DATABASE_URL=ws://localhost:8000    # SurrealDB connection
DATABASE_NAMESPACE=axel_tournament
DATABASE_NAME=axel_tournament
DATABASE_USER=root
DATABASE_PASS=root
MAX_CAPACITY=100                    # Max concurrent rooms + matches
MAX_CLAIM_DELAY_MS=1000            # Max delay at 100% capacity
```

## API Endpoints

- `GET /health` - Health check
- `GET /capacity` - Current server load statistics
- `GET /room/:room_id` - WebSocket upgrade for interactive games

## Running

```bash
cargo build --release
./target/release/judge
```

## Key Features

### Capacity Tracking

The server tracks active rooms and matches to calculate load:

- Load = (active_rooms + active_matches) / MAX_CAPACITY
- Claim delay = Load × MAX_CLAIM_DELAY_MS
- Ensures idle servers pick up work faster than busy servers

### Atomic Claiming

Uses database atomic operations to prevent duplicate execution:

```rust
// Try to claim match
let result = db.query(
    "UPDATE $match_id SET status='queued' WHERE status='pending' RETURN AFTER"
).await?;

if result.is_empty() {
    // Another server already claimed it
    continue;
}

// Successfully claimed - execute match
execute_match(match_record).await?;
```

### Generic Game Execution

The same `execute_game` function works for all player types:

```rust
async fn execute_game<G: Game>(
    game: &G,
    players: Vec<Box<dyn Player>>,  // Can be Docker or WebSocket
) -> GameResult {
    let mut state = game.initial_state();

    for round in 0..game.num_rounds() {
        // Collect moves from all players
        for (idx, player) in players.iter_mut().enumerate() {
            let state_str = game.encode_state_for_player(&state, idx);
            player.send_state(&state_str).await?;

            let move_str = player.receive_move(timeout_ms).await?;
            let parsed_move = game.parse_move(&move_str)?;

            state = game.apply_move(&state, idx, &parsed_move)?;
        }

        if game.is_game_over(&state) {
            break;
        }
    }

    GameResult { scores: game.get_scores(&state) }
}
```

## Future Work

- Implement WebSocket player fully (currently has TODO placeholders)
- Support for spectators
- Real-time match progress updates
- Game replay storage
- Multi-game server instances (currently only supports tic-tac-toe)
- Graceful shutdown (finish active games before stopping)
