# Game Communication Protocols

This directory contains the communication protocol specifications for each game supported by the judge server.

## General Protocol Rules

- All messages are text-based and sent over WebSocket connections
- Messages are case-sensitive unless otherwise specified
- Invalid moves result in immediate game termination with `WrongAnswer` result
- Timeout violations result in `TimeLimitExceeded` result
- Each player receives their own perspective of the game state

## Available Games

- [Rock Paper Scissors](rock-paper-scissors.md)
- [Prisoners Dilemma](prisoners-dilemma.md) 
- [Tic Tac Toe](tic-tac-toe.md)
