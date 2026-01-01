# Prisoner's Dilemma Game Server

Game server for the classic game theory Prisoner's Dilemma.

## Game Description

A two-player game theory scenario where players must choose to cooperate or defect. Points are awarded based on both players' choices:
- Both cooperate: 3 points each
- Both defect: 1 point each
- One cooperates, one defects: Defector gets 5 points, cooperator gets 0 points

## Features

- Automated bot-vs-bot matches over 100 rounds
- WebSocket support for interactive gameplay
- Supports Rust, Go, and C player submissions
- Docker-isolated player code execution

## Configuration

- **Game ID**: `prisoners-dilemma`
- **Port**: 8083
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

The game server is deployed as a containerized service and automatically watches the database for pending matches with `game_id = "prisoners-dilemma"`.
