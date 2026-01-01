# Tic Tac Toe Game Server

Game server for the classic Tic Tac Toe game.

## Game Description

A two-player strategy game played on a 3x3 grid. Players take turns marking spaces with X or O. The first player to get three in a row (horizontally, vertically, or diagonally) wins.

## Features

- Interactive WebSocket gameplay for human players
- Automated bot-vs-bot matches
- Supports Rust, Go, and C player submissions
- Docker-isolated player code execution
- Real-time game state updates

## Configuration

- **Game ID**: `tic-tac-toe`
- **Port**: 8084
- **Rounds per match**: 1
- **Turn timeout**: 30000ms (30 seconds for interactive play)
- **Memory limit**: 64MB
- **CPU limit**: 1.0 core

## Get Started

```bash
cargo run
```

## Environment Variables

- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection for match management

## Deployment

The game server is deployed as a containerized service and automatically watches the database for pending matches with `game_id = "tic-tac-toe"`. It also provides WebSocket endpoints for interactive gameplay.
