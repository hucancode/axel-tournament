# Axel Tournament

A tournament system where users submit code to play a multiplayer game to compete with other players.

Tech stack:
- SurrealDB
- Rust, Axum, Docker
- TypeScript, Svelte
- Terraform, Kubernetes, AWS

## Services

### api
Backend service that powers the platform.
- Authentication with JWT + Google OAuth, password reset, and role-based access
- User profiles and location metadata
- Game, tournament, match, and leaderboard management
- Code submission handling for multiple languages
- Admin endpoints for moderation and scheduling

### web
SvelteKit frontend for the player and admin experience.
- Sign in, registration, and profile management
- Tournament browsing, registration, and submissions
- Leaderboards and match visibility
- Admin dashboard for users, games, and tournaments

### healer
Background service that keeps matches healthy in the database.
- Watches pending/running matches via LIVE queries
- Refreshes stale pending matches
- Re-queues stale running matches back to pending
- Configurable intervals and staleness thresholds via environment variables

### judge
Match runner and results reporter.
- Listens for pending matches and claims them atomically
- Builds a workspace with game server code + player submissions
- Executes matches inside a sandboxed Docker container (CPU/memory/network limits)
- Parses results, reports scores/errors, and updates tournament totals

## Makefile

The repository includes a Makefile with helper targets.

### Targets
- `make test-db-up`: Start a local in-memory SurrealDB container for testing
- `make test-db-down`: Stop the local test database container

### Configuration
Override defaults via environment variables when needed:
- `CONTAINER_RUNTIME` (default: `docker`)
- `DATABASE_PORT` (default: `8000`)

Example:
```bash
CONTAINER_RUNTIME=podman DATABASE_PORT=9000 make test-db-up
```
