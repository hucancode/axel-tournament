# Axel Tournament Healer

Service that keeps match records healthy.

## Features
- Monitors pending and running matches
- Refreshes stale pending matches
- Re-queues stale running matches for retry
- Runs continuously with configurable intervals

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
