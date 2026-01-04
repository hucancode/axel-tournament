import { Graphics, Text } from 'pixi.js';
import { BasePixiGame } from './BasePixiGame';
import { COLORS, parseMessage } from './types';

export class TicTacToeGame extends BasePixiGame {
  private board: (string | null)[] = Array(9).fill(null);
  private mySymbol: 'X' | 'O' | null = null;
  private isMyTurn = false;
  private cells: Graphics[] = [];
  private finalScore: string | null = null;

  public handleMessage(data: string): void {
    const parts = parseMessage(data);
    if (!parts.length) return;

    switch (parts[0]) {
      case 'START':
        if (parts.length === 2) {
          this.mySymbol = parts[1] as 'X' | 'O';
          this.gameState.status = 'playing';
          this.isMyTurn = this.mySymbol === 'X';
          this.render();
        } else {
          console.warn('TicTacToe: START message missing player symbol, waiting for BOARD/YOUR_TURN');
          this.gameState.status = 'playing';
          this.render();
        }
        break;
      case 'BOARD':
        // Server sends: "BOARD X.O.X...." (9 characters after BOARD prefix)
        if (parts.length === 2) {
          this.parseBoardState(parts[1]);
          this.isMyTurn = false; // Wait for explicit YOUR_TURN message from server
          this.render();
        }
        break;
      case 'YOUR_TURN':
        this.isMyTurn = true;
        // If we don't know our symbol yet, infer it from being first to move
        if (!this.mySymbol && this.isMyTurn) {
          this.mySymbol = 'X'; // X always goes first in tic-tac-toe
          console.log('TicTacToe: Inferred player symbol as X (first mover)');
        }
        this.render();
        break;
      case 'SCORE':
        if (parts.length === 2) {
          this.finalScore = parts[1];
          this.gameState.status = 'finished';
          this.gameState.result = `Final Score: ${parts[1]}`;
          this.render();
        }
        break;
      case 'END':
        this.gameState.status = 'finished';
        this.render();
        break;
    }
  }

  protected render(): void {
    this.container.removeChildren();
    
    // Draw board
    for (let i = 0; i < 9; i++) {
      const row = Math.floor(i / 3);
      const col = i % 3;
      const x = col * 120 + 40;
      const y = row * 120 + 80;
      
      const cell = new Graphics();
      cell.rect(x, y, 100, 100);
      cell.fill(COLORS.WHITE);
      cell.stroke({ width: 2, color: COLORS.BLACK });
      
      if (this.gameState.status === 'playing' && !this.board[i] && this.isMyTurn) {
        cell.interactive = true;
        cell.cursor = 'pointer';
        cell.on('pointerdown', () => this.makeMove(row, col));
      }
      
      this.container.addChild(cell);
      this.cells[i] = cell;
      
      // Draw symbol
      if (this.board[i]) {
        const text = new Text({
          text: this.board[i]!,
          style: { fontSize: 48, fill: COLORS.BLACK }
        });
        text.x = x + 50 - text.width / 2;
        text.y = y + 50 - text.height / 2;
        this.container.addChild(text);
      }
    }
    
    // Status text
    const status = new Text({
      text: this.getStatusText(),
      style: { fontSize: 16, fill: COLORS.BLACK }
    });
    status.x = 200 - status.width / 2;
    status.y = 20;
    this.container.addChild(status);
  }

  private makeMove(row: number, col: number): void {
    if (!this.wsConnected || this.gameState.status !== 'playing' || !this.isMyTurn) return;
    
    const index = row * 3 + col;
    if (this.board[index]) return;
    
    this.sendMessage(`MOVE ${row} ${col}`);
    this.board[index] = this.mySymbol;
    this.isMyTurn = false;
    this.render();
  }

  private parseBoardState(data: string): void {
    const trimmed = data.trim();

    // Handle board with newlines (3 lines format)
    if (trimmed.includes('\n')) {
      const lines = trimmed.split('\n');
      if (lines.length === 3) {
        for (let row = 0; row < 3; row++) {
          for (let col = 0; col < 3; col++) {
            const char = lines[row][col];
            const index = row * 3 + col;
            this.board[index] = char === '.' ? null : char;
          }
        }
      }
    } else {
      // Handle single-line board format (9 characters)
      if (trimmed.length === 9) {
        for (let i = 0; i < 9; i++) {
          const char = trimmed[i];
          this.board[i] = char === '.' ? null : char;
        }
      }
    }
  }

  private getStatusText(): string {
    if (this.gameState.status === 'waiting') return 'Waiting for game...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    return this.isMyTurn ? `Your turn (${this.mySymbol})` : "Opponent's turn";
  }
}
