# Axel Tournament Healer

Background service to trigger match runner and recover from errors.

## Features
- Monitors pending and running matches
- Refreshes stale pending matches
- Re-queues stale running matches for retry

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
