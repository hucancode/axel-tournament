# Axel Tournament Judge

Service that runs matches and reports results.

## Features
- Claims pending matches and executes them
- Runs games in an isolated sandbox
- Collects scores and match metadata
- Reports results and updates tournament totals
- Handles match failures and error states

## Get started
Run `make sandbox-image` if you haven't done so
```bash
# cp .env.example .env
cargo run
# or
cargo test
```

## Environment variables
- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection
- `SANDBOX_IMAGE`: sandbox image name/tag
- `JUDGE_WORKSPACE_DIR`: workspace root for match runs
