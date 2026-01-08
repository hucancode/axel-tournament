# Judge Server

A single, scalable judge server that executes and validates games (Tic-Tac-Toe, Rock-Paper-Scissors, Prisoner's Dilemma) with both automated (Docker) and interactive (WebSocket) player implementations.

## Sandbox Architecture

The judge uses a **layered security approach** for different execution stages:

### Compilation (Cgroups Only)
- **Compiler execution**: Trusted, only needs resource limits (CPU/memory via cgroups)
- **No namespace isolation**: Allows rustup to work normally, simpler setup
- **Output**: Dynamically linked binaries (system libraries available in execution sandbox)
- **Prevents**: Resource exhaustion attacks (infinite loops, memory bombs in compiler)

### Bot Execution (Full Isolation)
- **User code execution**: Untrusted, needs full isolation
- **Namespace isolation**: PID, Mount, User, IPC, UTS namespaces
- **Filesystem isolation**: pivot_root with minimal rootfs
- **Additional security**: Landlock, Seccomp, capability dropping
- **Resource limits**: Cgroups for CPU/memory/PID limits
- **Prevents**: Filesystem access, network access, privilege escalation, resource exhaustion

## Architecture

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

## Useful Commands

Cleanup leaked user submission process (they are not supposed to be leaked but things happens while testing)
```
sudo pkill -9 -f "test_submission"
```
Check for leaked cgroup
```
sudo find /sys/fs/cgroup/judge -type d
# delete them with
sudo find /sys/fs/cgroup/judge -depth -type d -exec rmdir {} \;
```
