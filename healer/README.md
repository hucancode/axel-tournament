# Axel Tournament Healer

Background service that compiles player submissions and monitors match health.

## Features
- Compiles player code submissions into Docker images
- Monitors pending and running matches
- Refreshes stale pending matches
- Re-queues stale running matches for retry
- Works with game servers maintained by developers

## Architecture

The healer service works with the new architecture where games are maintained by developers. It compiles player submissions and ensures matches are properly executed by the dedicated game servers (prisoners-dilemma, rock-paper-scissors, tic-tac-toe).

## Get started
```bash
# cp .env.example .env
cargo run
# or
cargo test
```

## Environment variables
- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection
- `HEALER_PENDING_STALE_SECONDS`: pending match staleness threshold
- `HEALER_RUNNING_STALE_SECONDS`: running match staleness threshold
- `HEALER_SWEEP_INTERVAL_SECONDS`: sweep interval
