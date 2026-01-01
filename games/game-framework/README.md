# Game Framework

Core framework for building game servers in the Axel Tournament platform.

## Overview

This framework provides the infrastructure for developers to create new game servers. Each game server runs independently and handles both automated bot matches and interactive WebSocket gameplay.

## Features

- **Match Watcher**: Monitors database for pending matches and executes them
- **Automated Match Execution**: Runs bot-vs-bot matches in Docker containers
- **Interactive WebSocket Rooms**: Real-time gameplay for human players
- **Docker Player Isolation**: Sandboxed execution of player code
- **Database Integration**: Direct connection to SurrealDB for match management

## Architecture

Game servers are developed and maintained by developers, not app users. Each game:
- Defines its own game logic by implementing the `Game` trait
- Runs as a standalone service with its own port
- Watches the database for matches to execute
- Provides WebSocket endpoints for interactive play
- Manages player code execution in isolated Docker containers

## Creating a New Game

1. Create a new Rust project in the `games/` directory
2. Add `game-framework` as a dependency
3. Implement the `Game` trait for your game logic
4. Configure the game metadata (ID, name, rules, timeouts, etc.)
5. Run the HTTP server and match watcher in `main.rs`

See existing games (prisoners-dilemma, rock-paper-scissors, tic-tac-toe) for examples.

## Environment Variables

- `DATABASE_URL`, `DATABASE_USER`, `DATABASE_PASS`, `DATABASE_NS`, `DATABASE_DB`: SurrealDB connection

## Game Trait

Implement these methods for your game:
- `new()`: Initialize game state
- `get_metadata()`: Return game configuration
- `process_turn()`: Handle player moves for automated mode
- `make_move()`: Process moves for interactive mode
- `is_finished()`: Check if game is complete
- `get_winner()`: Determine game outcome
