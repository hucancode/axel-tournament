# Room Management Protocol

## Overview

This document describes the room management protocol for interactive human vs human games.

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
const room = await roomService.create({
  name: "My Room",
  game_id: "tic-tac-toe",
  human_timeout_ms: 30000
});

// Request: POST /api/rooms
// Headers: Authorization: Bearer <jwt>
// Body: { "name": "My Room", "game_id": "tic-tac-toe", "human_timeout_ms": 30000 }
// Response: { "id": "room_xyz", "game_id": "tic-tac-toe", ... }
```

### 2. Connect WebSocket (Join happens automatically)
```typescript
// Navigate to room page, then connect WebSocket
const socket = new RoomSocket(room.game_id, room.id);
await socket.connect();

// WebSocket URL: ws://judge/ws/tic-tac-toe/room_xyz
```

### 3. Authenticate via LOGIN
```
Client → Server: LOGIN {jwt_token}
```

Client must send LOGIN within 10 seconds. The server:
1. Validates JWT
2. Joins player to room (if not already in)
3. Connects WebSocket to player

**Responses:**
- `LOGIN_OK {player_id}` - New connection
- `LOGIN_OK {player_id} RECONNECT` - Reconnecting after disconnect
- `LOGIN_FAILED {error}` - Authentication failed

### 4. Play Game
```
// Host starts game
Client → Server: START
Server → All: GAME_STARTED

// Game messages
Server → Player1: START X
Server → Player2: START O
Server → All: BOARD .........
Server → All: TURN 0
Server → Player1: YOUR_TURN

// Player moves
Player1 → Server: MOVE 1 1
Server → All: BOARD ....X....
```

### 5. Chat
```
Client → Server: CHAT Hello!
Server → All: CHAT user:abc Hello!
```

### 6. Reconnection
```
// Player reconnects after disconnect
Client → Server: LOGIN {jwt_token}
Server → Client: LOGIN_OK user:abc RECONNECT
Server → Client: REPLAY_START
Server → Client: PLAYER_JOINED user:def
Server → Client: GAME_STARTED
Server → Client: START O
Server → Client: BOARD ....X....
Server → Client: REPLAY_END
```

### 7. Leave Room
```
Client → Server: LEAVE
Server → Client: LEFT_ROOM
Server → Others: PLAYER_LEFT user:abc
```

## Protocol Reference

### HTTP Endpoints

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | `/api/rooms` | JWT | Create room |
| GET | `/api/rooms` | - | List waiting rooms |
| GET | `/api/rooms/{id}` | - | Get room details |

### WebSocket Messages

**URL:** `/ws/{game_id}/{room_id}`

**Client → Server:**
- `LOGIN {jwt_token}` - Authenticate and join (must be first message)
- `START` - Start game (host only)
- `CHAT {message}` - Send chat
- `LEAVE` - Leave room
- Game-specific moves (e.g., `MOVE 1 1`)

**Server → Client:**
- `LOGIN_OK {player_id}` - Connected
- `LOGIN_OK {player_id} RECONNECT` - Reconnecting
- `LOGIN_FAILED {error}` - Auth failed
- `REPLAY_START` / `REPLAY_END` - History replay bounds
- `PLAYER_JOINED {user_id}` - Player joined
- `PLAYER_LEFT {user_id}` - Player left
- `LEFT_ROOM` - Leave confirmed
- `HOST_CHANGED {user_id}` - New host
- `GAME_STARTED` - Game started
- `GAME_FINISHED {results}` - Game finished
- `CHAT {user_id} {message}` - Chat message
- `ERROR {message}` - Error
- Game messages: `START`, `BOARD`, `TURN`, `YOUR_TURN`, `SCORE`, `END`

## Room States
- `waiting` - Accepting players
- `playing` - Game in progress
- `finished` - Game completed

## Player States
- Connected: In `players` with active WebSocket
- Disconnected: In `players` but no WebSocket (can reconnect)
- Left: Removed from `players` (explicit LEAVE)

## Host Transfer
1. Host disconnects/leaves → Transfers to next connected player
2. Original host reconnects → Does NOT regain host
3. All players leave → Room deleted
