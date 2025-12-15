# Axel Tournament

A tournament system where users submit code to play a multiplayer game to compete with other players.

## Features

### Authentication & Authorization
- User registration and login with secure password hashing (Argon2)
- JWT-based authentication
- OAuth2 Google login integration
- Password reset via email
- Role-based access control (Admin/Player)
- User ban system

### User Management
- User profiles with geographical location (ISO country codes)
- Location defaults to US
- Profile management endpoints

### Code Submissions
- Support for multiple programming languages (Rust, Go, C)
- Code upload and storage
- Submission history per tournament
- File-based code storage for processing by separate match-running service

### Tournament System
- Create and manage tournaments
- Each tournament belongs to one game
- Minimum 2 players to start
- Tournament status management (Scheduled, Registration, Running, Completed, Cancelled)
- Join/leave tournament functionality
- Participant tracking and scoring

### Game Management
- Define games with custom rules (JSON format)
- Specify supported programming languages per game
- Active/inactive game status

### Match Management
- A match is a single event happens between often 2 (sometimes more) players
- A match must specify players and which game they play, at what time, in which tournament
- A match go through phases below:
  - preparing, accepting players
  - on-going, players are actively playing
  - finished, the match is finished, not accepting any player input, outcome are in process
  - over, the result are ready 

### Leaderboard
- Top K players by score
- Filter by tournament or game
- Global leaderboard support
- Real-time ranking

### Admin Features
- Create and manage games
- Create and manage tournaments
- Ban/unban players
- View all users
- Tournament scheduling

## Getting Started

```bash
cp .env.example .env
docker-compose up -d
```

The API will be available at `http://localhost:8080`

## Authentication

Protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

## Example Requests

### Register a User
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "player@example.com",
    "username": "player1",
    "password": "securepass123",
    "location": "US"
  }'
```

### Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "player@example.com",
    "password": "securepass123"
  }'
```

### Create a Game (Admin)
```bash
curl -X POST http://localhost:8080/api/admin/games \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{
    "name": "Battle Arena",
    "description": "A competitive battle arena game",
    "rules": {
      "max_rounds": 100,
      "time_limit": 60
    },
    "supported_languages": ["rust", "go", "c"]
  }'
```

### Create a Tournament (Admin)
```bash
curl -X POST http://localhost:8080/api/admin/tournaments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{
    "game_id": "game:123",
    "name": "Spring Championship",
    "description": "Quarterly tournament",
    "min_players": 2,
    "max_players": 200,
    "start_time": "2025-04-01T00:00:00Z",
    "end_time": "2025-04-07T23:59:59Z"
  }'
```

### Submit Code
```bash
curl -X POST http://localhost:8080/api/submissions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <player-token>" \
  -d '{
    "tournament_id": "tournament:456",
    "language": "rust",
    "code": "fn main() { println!(\"Hello!\"); }"
  }'
```

### Get Leaderboard
```bash
curl http://localhost:8080/api/leaderboard?limit=10
```
