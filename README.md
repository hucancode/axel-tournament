# Axel Tournament

A tournament system where users submit code to play a multiplayer game to compete with other players.

Tech stack:
- SurrealDB
- Rust, Axum, Docker
- TypeScript, Svelte, Storybook
- Terraform, Kubernetes, AWS

## Services

*Use the following script to quickly spin up test DB on your local machine*
- `make test-db-up`: Start a local in-memory SurrealDB container for testing
- `make test-db-down`: Stop the local test database container

*You also need to run `make sandbox-image` to build sandbox image on your local machine before running `judge` server*

### api
Backend service that powers the platform.
- Authentication with JWT + Google OAuth, password reset, and role-based access
- Game, tournament, match, and leaderboard management
- Code submission handling
- Admin endpoints for moderation

### web
Web frontend for the players
- Sign in, registration, and profile management
- Tournament browsing, registration, and submissions
- Leaderboards
- Admin dashboard

### healer
Background service to trigger match runner.
- Refreshes stale pending matches
- Re-queues stale running matches back to pending

### judge
Match runner and results reporter.
- Listens for pending matches and claims them atomically
- Builds a workspace with game server code + player submissions
- Executes matches inside a sandboxed Docker container (CPU/memory/network limits)
- Parses results, reports scores/errors, and updates tournament totals
