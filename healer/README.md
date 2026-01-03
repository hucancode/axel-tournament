# Axel Tournament Healer

Background service that monitors match health and manages the match queue.

## Features
- Monitors pending and running matches
- Refreshes stale queued matches back to pending
- Re-queues stale running matches for retry
- Ensures match queue stays healthy and responsive

## Get started
```bash
# cp .env.example .env
cargo run
# or
cargo test
```

## Environment variables
- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection
