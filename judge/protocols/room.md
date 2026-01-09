# Room Management Protocol

## Overview

This document describes the complete room management protocol for interactive human vs human games.

## Architecture

```
┌─────────┐                                    ┌─────────────┐
│ Client  │ ──── HTTP (JWT auth) ────────────► │ Load Balancer│
└─────────┘                                    └──────┬───────┘
     │                                                │
     │                                         ┌──────▼──────┐
     │                                         │ Sticky Hash │
     │                                         │  by room_id │
     │                                         └──────┬───────┘
     │                                                │
     │  ┌──────────────────────────┬──────────────────┴────────────┐
     │  │                          │                               │
     ▼  ▼                          ▼                               ▼
┌──────────┐                  ┌──────────┐                   ┌──────────┐
│ Judge A  │                  │ Judge B  │                   │ Judge C  │
│ (rooms   │                  │ (rooms   │                   │ (rooms   │
│  1,4,7)  │                  │  2,5,8)  │                   │  3,6,9)  │
└──────────┘                  └──────────┘                   └──────────┘
```

## Flow

### 1. Create Room (HTTP)
```typescript
// Client
const room = await roomService.create({
  name: "My Room",
  game_id: "tic-tac-toe",
  human_timeout_ms: 30000
});

// Request: POST /api/rooms
// Headers: Authorization: Bearer <jwt>
// Body: {
//   "name": "My Room",
//   "game_id": "tic-tac-toe",
//   "host_id": "user:abc",
//   "host_username": "Alice",
//   "human_timeout_ms": 30000
// }

// Response: {
//   "id": "room_xyz",
//   "game_id": "tic-tac-toe",
//   "name": "My Room",
//   "host_id": "user:abc",
//   "host_username": "Alice",
//   "players": [],
//   "status": "waiting",
//   "max_players": 2
// }
```

### 2. Join Room (HTTP)
```typescript
// Client
const room = await roomService.join(roomId);

// Request: POST /api/rooms/{room_id}/join
// Headers: Authorization: Bearer <jwt>

// Response: Same as create, with updated players list
```

### 3. Connect WebSocket
```typescript
// Client gets player_id and JWT token
const playerId = getCurrentUser().id;  // e.g., "user:def"
const token = getAuthToken();  // JWT token

const socket = new RoomSocket(room.game_id, room.id);
await socket.connect();

// WebSocket URL: ws://judge/ws/tic-tac-toe/room_xyz
```

### 4. Client Authenticates
```
Client → Server: LOGIN {jwt_token}
```

Client must send LOGIN message with JWT token as the first message after WebSocket connection (within 10 second timeout).

### 5. Server Sends Connection Confirmation

**For new connections:**
```
Server → Client: LOGIN_OK {player_id}
```

**For reconnections:**
```
Server → Client: LOGIN_OK {player_id} RECONNECT
```

**On authentication failure:**
```
Server → Client: LOGIN_FAILED {error_message}
```

Possible error messages:
- `LOGIN_FAILED Login timeout` - Client didn't send LOGIN within 10 seconds
- `LOGIN_FAILED Missing LOGIN command` - First message wasn't LOGIN
- `LOGIN_FAILED {jwt_error}` - JWT validation failed
- `LOGIN_FAILED Invalid player_id format` - player_id from JWT is malformed

### 6. Play Game
```typescript
// Host starts game
socket.on('connected', () => {
  if (isHost) {
    socket.startGame();
  }
});

// Server broadcasts to all
Server → All: GAME_STARTED

// Game logic sends START messages
Server → Player1: START X
Server → Player2: START O

// Game messages
Server → All: BOARD .........
Server → All: TURN 0
Server → Player1: YOUR_TURN

// Player sends move
Player1 → Server: MOVE 1 1

// Game broadcasts updated state
Server → All: BOARD ...X.....
Server → All: TURN 1
```

### 7. Chat
```typescript
socket.chat("Good luck!");

// Client → Server: CHAT Good luck!
// Server → All: CHAT user:def Good luck!
```

Note: The server broadcasts chat messages with the player_id only (no username). Clients should look up the username from the player_id.

### 8. Disconnection & Reconnection

**Player disconnects:**
```
Server detects disconnect
Server → Remaining: PLAYER_LEFT user:def
Server marks user:def as disconnected (stays in player_ids)
```

**Player reconnects:**
```typescript
// 1. Join again via HTTP
const room = await roomService.join(roomId);  // Same room

// 2. Connect WebSocket and authenticate
const socket = new RoomSocket(room.game_id, room.id);
await socket.connect();
Client → Server: LOGIN {jwt_token}

// 3. Server detects reconnection and sends combined LOGIN_OK
Server → Player: LOGIN_OK user:def RECONNECT
Server → Player: REPLAY_START

// 4. Server replays room history
Server → Player: PLAYER_JOINED user:abc
Server → Player: GAME_STARTED
Server → Player: CHAT user:abc Hello

// 5. Server replays game state
Server → Player: START O
Server → Player: BOARD ...X.....
Server → Player: TURN 1
Server → Player: YOUR_TURN

Server → Player: REPLAY_END
```

### 9. Leave Room
```typescript
socket.leave();

// Client → Server: LEAVE
// Server → Client: LEFT_ROOM
// Server removes player from player_ids permanently
// Server → Remaining Players: PLAYER_LEFT user:def
// If host left: HOST_CHANGED user:abc
```

## Protocol Reference

### HTTP Endpoints (Judge Server)

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | `/api/rooms` | JWT | Create room |
| GET | `/api/rooms` | - | List waiting rooms |
| GET | `/api/rooms/{id}` | - | Get room details |
| POST | `/api/rooms/{id}/join` | JWT | Join room |
| DELETE | `/api/rooms/{id}/leave` | JWT | Leave room |

### WebSocket Messages

**URL:** `/ws/{game_id}/{room_id}`

Examples:
- `/ws/tic-tac-toe/room_xyz`
- `/ws/rock-paper-scissors/room_xyz`
- `/ws/prisoners-dilemma/room_xyz`

**Client → Server:**
- `LOGIN {jwt_token}` - Authenticate (must be first message within 10 seconds)
- `START` - Start game (host only)
- `CHAT {message}` - Send chat
- `LEAVE` - Leave room
- Game-specific moves

**Server → Client:**
- `LOGIN_OK {player_id}` - Successfully connected (new connection)
- `LOGIN_OK {player_id} RECONNECT` - Reconnecting (was disconnected)
- `LOGIN_FAILED {error_message}` - Authentication failed
- `REPLAY_START` - Start of message replay (reconnection only)
- `REPLAY_END` - End of message replay (reconnection only)
- `PLAYER_JOINED {user_id}` - Player joined
- `PLAYER_LEFT {user_id}` - Player left
- `LEFT_ROOM` - Confirmation of successful leave
- `HOST_CHANGED {user_id}` - New host assigned
- `GAME_STARTED` - Game started
- `GAME_FINISHED {results_json}` - Game finished
- `CHAT {user_id} {message}` - Chat message (no username, clients should look it up)
- `ERROR {message}` - Error occurred
- Game-specific messages (START, BOARD, TURN, YOUR_TURN, SCORE, END, etc.)

## State Management

### Room States
- `waiting` - Waiting for players
- `playing` - Game in progress
- `finished` - Game completed

### Player States
- In `players` and `connected_players[i] == Some(HumanPlayer)` = Connected
- In `players` and `connected_players[i] == None` = Disconnected temporarily
- Not in `players` = Left permanently

## Host Transfer Rules

1. **Host disconnects:** Transfers to next connected player immediately
2. **Host leaves (explicit LEAVE):** Transfers to next connected player immediately
3. **Reconnecting original host:** Does NOT regain host status
4. **All players explicit LEAVE:** Room is deleted immediately

## Bot Players vs Human Players

### Human Players (Interactive Rooms)
- Join via HTTP with JWT auth
- Connect via WebSocket
- Receive all room messages (PLAYER_JOINED, CHAT, etc.)
- Can reconnect after disconnect

### Bot Players (Automated Matches)
- Created directly in memory by match_watcher
- No HTTP join, no WebSocket, no auth
- Go straight to `game.run()` with bot Player implementations
- No room messages, no reconnection

Completely separate code paths - no overlap!

## Load Balancer Configuration

See `nginx-load-balancer.conf` for complete setup. Key points:

1. **API routes** → Round-robin to API servers
2. **Room routes** (`/api/rooms/*`) → Sticky hash to Judge servers
3. **WebSocket routes** (`/ws/*/*`) → Sticky hash to Judge servers
4. **Hash key:** `room_id` ensures all players in same room hit same Judge instance

## Example Client Usage

```typescript
import { roomService } from '$lib/services/rooms';
import { RoomSocket } from '$lib/services/roomSocket';

// 1. Create or join room via HTTP
const room = await roomService.create({
  name: "My Game",
  game_id: "tic-tac-toe"
});

// OR
const room = await roomService.join(roomId);

// 2. Connect WebSocket and authenticate
const playerId = roomService.getCurrentUser().id;
const token = roomService.getAuthToken();
const socket = new RoomSocket(room.game_id, room.id);

// Connect to WebSocket
await socket.connect();

// Send LOGIN message with JWT (must be first message within 10 seconds)
await socket.send(`LOGIN ${token}`);

// Handle connection confirmation
socket.on('login_ok', (playerId, isReconnecting) => {
  console.log('Connected to room:', playerId);
  if (isReconnecting) {
    console.log('Reconnecting...');
  }
});

socket.on('login_failed', (error) => {
  console.error('Login failed:', error);
});

socket.on('player_joined', (userId) => {
  console.log('Player joined:', userId);
  // Look up username from userId
});

socket.on('game_started', () => {
  console.log('Game started!');
});

socket.on('your_turn', () => {
  console.log('Your turn!');
  socket.sendMove('MOVE 1 1');
});

socket.on('chat', (userId, message) => {
  console.log(`${userId}: ${message}`);
  // Look up username from userId for display
});

// 3. Start game (if host)
if (room.host_id === playerId) {
  socket.startGame();
}

// 4. Chat
socket.chat('Hello everyone!');

// 5. Leave
socket.leave();
socket.on('left_room', () => {
  console.log('Successfully left room');
  socket.disconnect();
});
```

## Security

1. **JWT Authentication:**
   - All HTTP endpoints that modify state require JWT in Authorization header
   - WebSocket connections require JWT via LOGIN message (not in URL)
2. **Player Verification:** WebSocket validates JWT and checks player is authorized to join room
3. **Host Validation:** Only host can start game
4. **Token Security:** JWT never appears in URLs, only in HTTP headers or WebSocket messages

## Reconnection Guarantees

When a player reconnects:
1. Receives all room-level messages since game started
2. Receives current game state from GameLogic
3. Can continue playing from current position
4. Does NOT regain host if they were original host

## Testing

Run the judge server:
```bash
cd judge
cargo run
```

The server will start on `http://localhost:8081` with the following endpoints ready:
- HTTP room management: `/api/rooms/*`
- WebSocket: `/ws/{game_id}/{room_id}/{player_id}`
