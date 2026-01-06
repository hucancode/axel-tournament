# Axel Tournament

A tournament system where users submit code to play a multiplayer game to compete with other players.

Tech stack:
- TypeScript, Svelte, Storybook
- Rust, Axum, cgroup, namespace
- SurrealDB
- Terraform, Kubernetes, AWS

## Services

### web
Web frontend for the players
- Sign in, registration, and profile management
- Tournament browsing, registration, and submissions
- Leaderboards
- Admin dashboard

### api
Backend service that powers the platform.
- Authentication with JWT + Google OAuth, password reset, and role-based access
- Game, tournament, match, and leaderboard management
- Code submission handling
- Admin endpoints for moderation

### healer
Background service to trigger match runner.
- Refreshes stale pending matches
- Re-queues stale running matches back to pending

### judge
Match runner and results reporter.
- Listens for pending matches and claims them atomically
- Compile user submissions
- Executes matches inside a sandboxed environment (CPU/memory/network limits)
- Parses results, reports scores/errors, and updates tournament totals

## Useful commands

*Use the following script to quickly spin up test DB on your local machine*
- `make test-db-up`: Start a local in-memory SurrealDB container for testing
- `make test-db-down`: Stop the local test database container
