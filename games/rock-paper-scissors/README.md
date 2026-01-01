# Rock Paper Scissors Game Server

Game server for the classic Rock Paper Scissors game.

## Game Description

A two-player game where players choose rock, paper, or scissors:
- Rock beats scissors
- Scissors beats paper
- Paper beats rock
- Same choice results in a tie

## Features

- Automated bot-vs-bot matches over 100 rounds
- WebSocket support for interactive gameplay
- Supports Rust, Go, and C player submissions
- Docker-isolated player code execution

## Configuration

- **Game ID**: `rock-paper-scissors`
- **Port**: 8082
- **Rounds per match**: 100
- **Turn timeout**: 2000ms
- **Memory limit**: 64MB
- **CPU limit**: 0.5 cores

## Get Started

```bash
cargo run
```

## Environment Variables

- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection for match management

## Deployment

The game server is deployed as a containerized service and automatically watches the database for pending matches with `game_id = "rock-paper-scissors"`.
