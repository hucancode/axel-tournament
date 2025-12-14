# Axel Tournament API

A comprehensive web backend API for running code tournament competitions, built with Axum, SurrealDB, and Docker.

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
- Support for ~200 players per tournament (configurable)
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
- A match phase could be either be 
  - scheduled, not happening, but waiting to happen
  - opening, accepting registration
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

## Tech Stack

- **Web Framework**: Axum 0.7
- **Database**: SurrealDB 2.0
- **Authentication**: JWT, Argon2, OAuth2
- **Email**: Lettre
- **Validation**: validator
- **Serialization**: serde
- **Logging**: tracing
- **Containerization**: Docker, Docker Compose

## Project Structure

```
axel-tournament/
├── src/
│   ├── main.rs              # Application entry point, routing
│   ├── config.rs            # Configuration management
│   ├── db.rs                # Database connection and schema
│   ├── error.rs             # Error handling
│   ├── models/              # Data models
│   │   ├── user.rs
│   │   ├── game.rs
│   │   ├── tournament.rs
│   │   ├── submission.rs
│   │   └── leaderboard.rs
│   ├── services/            # Business logic
│   │   ├── auth.rs
│   │   ├── user.rs
│   │   ├── game.rs
│   │   ├── tournament.rs
│   │   ├── submission.rs
│   │   ├── leaderboard.rs
│   │   └── email.rs
│   ├── handlers/            # HTTP request handlers
│   │   ├── auth.rs
│   │   ├── user.rs
│   │   ├── game.rs
│   │   ├── tournament.rs
│   │   ├── submission.rs
│   │   ├── leaderboard.rs
│   │   └── admin.rs
│   └── middleware/          # Authentication middleware
│       └── auth.rs
├── Cargo.toml               # Rust dependencies
├── Dockerfile               # Multi-stage Docker build
├── docker-compose.yml       # Docker services configuration
└── .env.example             # Environment variables template

```

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Rust 1.75+ (for local development)

### Quick Start with Docker

1. Clone the repository and navigate to the project directory

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` and set your configuration (especially `JWT_SECRET` and `ADMIN_PASSWORD`)

4. Start the services:
```bash
docker-compose up -d
```

The API will be available at `http://localhost:8080`

### Seed Admin User

On first startup, if the user table is empty, the application will automatically create an admin user with credentials from environment variables:

- **Email**: `ADMIN_EMAIL` (default: admin@axel-tournament.com)
- **Password**: `ADMIN_PASSWORD` (must be set in .env)

**Important**: The seed admin user is only created if the database has no users. If you already have users in the database, no admin user will be automatically created.

You can use this admin account to:
- Create and manage games
- Create and manage tournaments
- Ban/unban users
- View all users

### Local Development

1. Install SurrealDB:
```bash
# Using Docker
docker run -d -p 8000:8000 surrealdb/surrealdb:latest start --log trace --user root --pass root file:/data/database.db
```

2. Copy and configure environment:
```bash
cp .env.example .env
# Edit .env with your settings
```

3. Build and run:
```bash
cargo build
cargo run
```

## API Endpoints

### Health Check
- `GET /health` - Health check endpoint

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login with email/password
- `POST /api/auth/reset-password` - Request password reset
- `POST /api/auth/confirm-reset` - Confirm password reset with token
- `GET /api/auth/google` - Get Google OAuth URL
- `GET /api/auth/google/callback` - Google OAuth callback

### User (Protected)
- `GET /api/users/profile` - Get current user profile
- `PATCH /api/users/location` - Update user location

### Games (Public reads, Admin writes)
- `GET /api/games` - List all active games
- `GET /api/games/:id` - Get game by ID
- `POST /api/admin/games` - Create game (Admin)
- `PUT /api/admin/games/:id` - Update game (Admin)
- `DELETE /api/admin/games/:id` - Delete game (Admin)

### Tournaments
- `GET /api/tournaments` - List tournaments (optional status filter)
- `GET /api/tournaments/:id` - Get tournament by ID
- `GET /api/tournaments/:id/participants` - Get tournament participants
- `POST /api/tournaments/:id/join` - Join tournament (Protected)
- `DELETE /api/tournaments/:id/leave` - Leave tournament (Protected)
- `POST /api/admin/tournaments` - Create tournament (Admin)
- `PATCH /api/admin/tournaments/:id` - Update tournament (Admin)

### Submissions (Protected)
- `POST /api/submissions` - Submit code for tournament
- `GET /api/submissions` - List user's submissions
- `GET /api/submissions/:id` - Get submission by ID

### Leaderboard (Public)
- `GET /api/leaderboard?limit=100&tournament_id=xxx&game_id=xxx` - Get leaderboard

### Admin
- `GET /api/admin/users` - List all users
- `POST /api/admin/users/:id/ban` - Ban user
- `POST /api/admin/users/:id/unban` - Unban user

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

## Configuration

All configuration is done via environment variables. See `.env.example` for all available options.

Key configurations:
- `JWT_SECRET`: Secret key for JWT signing (required, change in production)
- `ADMIN_EMAIL`: Email for seed admin user (default: admin@axel-tournament.com)
- `ADMIN_PASSWORD`: Password for seed admin user (required)
- `DATABASE_URL`: SurrealDB connection URL
- `GOOGLE_CLIENT_ID`: Google OAuth client ID
- `GOOGLE_CLIENT_SECRET`: Google OAuth client secret
- `SMTP_*`: Email configuration for password reset

## Database Schema

The application uses SurrealDB with the following main tables:
- `user`: User accounts and profiles
- `game`: Game definitions and rules
- `tournament`: Tournament instances
- `tournament_participant`: Tournament participation and scores
- `submission`: Code submissions

Schema is automatically initialized on first connection.

## Security Best Practices

1. Always change `JWT_SECRET` in production
2. Use strong, unique passwords
3. Enable HTTPS in production (configure reverse proxy)
4. Regularly rotate JWT secrets
5. Configure CORS properly for your frontend domain
6. Use environment-specific configurations
7. Keep dependencies updated

## OAuth2 Google Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable Google+ API
4. Create OAuth 2.0 credentials
5. Add authorized redirect URI: `http://localhost:8080/api/auth/google/callback` (adjust for production)
6. Copy Client ID and Client Secret to `.env`

## Future Integration Points

### Match Running Service
The system stores submitted code in the `uploads/` directory and maintains submission records in the database. A separate service should:
1. Poll for new submissions with status `pending`
2. Compile and execute submitted code
3. Run matches between participants
4. Update scores in `tournament_participant` table
5. Update submission status to `accepted` or `failed`

### Frontend Applications
Two frontend applications are planned:
1. **Player Frontend**: Tournament registration, code submission, leaderboard viewing
2. **Admin Dashboard**: Game/tournament management, user moderation, analytics

Both can connect via the RESTful API endpoints.

## Development

### Running Tests

#### Unit Tests (10 tests, no database required)
```bash
cargo test --test integration_tests
```

#### Full API Tests (9 tests with database)
The API tests require a SurrealDB instance running at `localhost:8001`. Use the Makefile to automatically start/stop the test database:

```bash
# Run all tests with automatic database management
make test

# Or manually:
make test-db              # Start test database
cargo test                # Run all tests
make test-db-stop         # Stop test database
```

**Makefile Commands:**
- `make test` - Run all tests with automatic test DB start/stop
- `make test-db` - Start SurrealDB test instance on port 8001
- `make test-db-stop` - Stop test database
- `make clean` - Clean build artifacts and stop test DB
- `make help` - Show all available commands

✅ **All 17 tests passing!** (9 unit + 8 API tests)

### Checking Code
```bash
cargo clippy
cargo fmt
```

### Building for Production
```bash
cargo build --release
```

## Troubleshooting

### Database Connection Issues
- Ensure SurrealDB is running and accessible
- Check `DATABASE_URL` in `.env`
- Verify network connectivity between services

### Authentication Errors
- Verify `JWT_SECRET` is set
- Check token expiration time
- Ensure Authorization header format is correct

### Email Not Sending
- Verify SMTP credentials
- Check firewall/network settings
- For development, emails are logged instead of sent (see `src/services/email.rs`)

## License

This project is provided as-is for tournament hosting purposes.

## Contributing

This is a tournament platform. For feature requests or bug reports, please open an issue.
