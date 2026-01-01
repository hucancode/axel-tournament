# Axel Tournament API Server

Backend API for the tournament platform.

## Features
- User authentication and access control
- Player profiles and account management
- Game catalog (games are maintained by developers, not users)
- Tournament lifecycle management
- Match scheduling and scoring
- Code submissions for multiple languages
- Leaderboards and rankings
- Admin tools for moderation and management

## Architecture

Games are now hardcoded in the API as static metadata. Game servers (prisoners-dilemma, rock-paper-scissors, tic-tac-toe) are developed and maintained separately by developers. The API provides endpoints to list available games, but game creation/modification is no longer done through the API.

## Get started
```bash
# cp .env.example .env
cargo run
# or
cargo test
```

## Environment variables
- `SERVER_HOST`, `SERVER_PORT`: API bind address and port
- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection
- `JWT_SECRET`, `JWT_EXPIRATION`: auth token signing and expiry
- `ADMIN_EMAIL`, `ADMIN_PASSWORD`: bootstrap admin credentials
- `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI`: Google OAuth settings
- `OAUTH_COOKIE_SECURE`, `OAUTH_STATE_TTL_SECONDS`: OAuth flow options
- `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `EMAIL_FROM`: email sender settings
- `DEFAULT_LOCATION`: default user location code
- `RUST_LOG`: logging level
