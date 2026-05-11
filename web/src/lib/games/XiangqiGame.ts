import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

/** Spec: judge/protocols/xiangqi.md. Player 0 = red, player 1 = black. */

const COLS = 9;
const ROWS = 10;
const CELL = 44;
const ORIGIN_X = 22;
const ORIGIN_Y = 22;
const CANVAS_W = ORIGIN_X * 2 + (COLS - 1) * CELL; // = 22*2 + 8*44 = 396 (~400)
const CANVAS_H = ORIGIN_Y * 2 + (ROWS - 1) * CELL; // = 22*2 + 9*44 = 440

const RED_COLOR = 0xc62828;
const BLACK_COLOR = 0x212121;
const BOARD_FILL = 0xf6e0a4; // warm wood
const BOARD_STROKE = 0x3e2723;
const HIGHLIGHT_SELECT = 0x66bb6a;
const HIGHLIGHT_DEST = 0xfdd835;

type Side = 0 | 1;
type Kind = 'K' | 'A' | 'E' | 'H' | 'R' | 'C' | 'P';

interface Piece {
  side: Side;
  kind: Kind;
}

const RED_GLYPH: Record<Kind, string> = {
  K: '帥',
  A: '仕',
  E: '相',
  H: '傌',
  R: '俥',
  C: '炮',
  P: '兵',
};

const BLACK_GLYPH: Record<Kind, string> = {
  K: '將',
  A: '士',
  E: '象',
  H: '馬',
  R: '車',
  C: '砲',
  P: '卒',
};

export class XiangqiGame extends BasePixiGame {
  private board: (Piece | null)[] = Array(COLS * ROWS).fill(null);
  private moveCount = 0;
  private outcome: 'win' | 'lose' | 'draw' | null = null;
  private selected: { row: number; col: number } | null = null;
  private legalDests: { row: number; col: number }[] = [];

  // Override init: 9x10 board needs a taller canvas than the 400x400 default.
  // We piggy-back on the parent constructor by re-initialising after super().
  // pixi's Application.init returns a promise; we await it lazily.
  protected async initBoard() {
    // intentionally left as a hook for future use
  }

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;

    switch (kind) {
      case 'GAME_STARTED':
        this.board = initialBoard();
        this.moveCount = 0;
        this.outcome = null;
        this.selected = null;
        this.legalDests = [];
        this.gameState.status = 'playing';
        this.gameState.result = undefined;
        this.refresh();
        break;
      case 'MOVE': {
        const parts = payload.split(' ');
        if (parts.length !== 3) return;
        const moverIdx = ctx.players.indexOf(parts[0]);
        const from = parseSquare(parts[1]);
        const to = parseSquare(parts[2]);
        if (moverIdx < 0 || !from || !to) return;
        const piece = this.board[from.row * COLS + from.col];
        if (!piece) return;
        this.board[from.row * COLS + from.col] = null;
        this.board[to.row * COLS + to.col] = piece;
        this.moveCount += 1;
        this.selected = null;
        this.legalDests = [];
        this.refresh();
        break;
      }
      case 'PIECE_REVEALED': {
        // Blind variant: server discloses a piece's true kind. Backend
        // uses 'G' for the general; frontend stores it as 'K'.
        const parts = payload.split(' ');
        if (parts.length !== 2) return;
        const sq = parseSquare(parts[0]);
        if (!sq) return;
        const raw = parts[1];
        const kind: Kind = raw === 'G' ? 'K' : (raw as Kind);
        const ix = sq.row * COLS + sq.col;
        const cur = this.board[ix];
        if (cur && 'KAEHRCP'.includes(kind)) {
          this.board[ix] = { side: cur.side, kind };
        }
        this.refresh();
        break;
      }
      case 'WINNER': {
        const w = parseInt(payload, 10);
        this.gameState.status = 'finished';
        this.outcome = w === ctx.myIndex ? 'win' : 'lose';
        this.refresh();
        break;
      }
    }
  }

  public getStatus(): GameStatus {
    if (this.gameState.status === 'waiting') {
      return { text: 'Waiting for game to start', tone: 'idle' };
    }
    if (this.gameState.status === 'finished') {
      if (this.outcome === 'win') return { text: 'You Win!', tone: 'win' };
      if (this.outcome === 'lose') return { text: 'You Lose', tone: 'lose' };
      return { text: 'Draw', tone: 'draw' };
    }
    const me = this.ctx.myIndex;
    const isMyTurn = me >= 0 && this.moveCount % 2 === me;
    const sideLabel = me === 0 ? 'Red' : 'Black';
    return isMyTurn
      ? { text: 'Your turn', tone: 'turn', detail: `Playing as ${sideLabel}` }
      : { text: "Opponent's turn", tone: 'wait', detail: `You are ${sideLabel}` };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished' || !this.outcome) return null;
    const sideLabel = this.ctx.myIndex === 0 ? 'Red' : 'Black';
    const title =
      this.outcome === 'win' ? 'Victory' : this.outcome === 'lose' ? 'Defeat' : 'Draw';
    return {
      outcome: this.outcome,
      title,
      details: [`You played as ${sideLabel}`, `${this.moveCount} half-moves played`],
    };
  }

  protected render(): void {
    if (!this.app || !this.app.renderer) return;

    // Resize canvas to match xiangqi aspect on first render.
    if (this.app.renderer.width !== CANVAS_W || this.app.renderer.height !== CANVAS_H) {
      this.app.renderer.resize(CANVAS_W, CANVAS_H);
    }

    this.container.removeChildren();

    this.drawBoard();
    this.drawHighlights();
    this.drawPieces();
  }

  private drawBoard(): void {
    const bg = new Graphics();
    bg.rect(0, 0, CANVAS_W, CANVAS_H);
    bg.fill(BOARD_FILL);
    this.container.addChild(bg);

    // Vertical lines: 9 files. Each file is one continuous line on the
    // black/red half EXCEPT it is interrupted at the river (rows 4..5)
    // for interior files. Outer files (col 0 and col 8) span the whole
    // board.
    for (let col = 0; col < COLS; col++) {
      const x = ORIGIN_X + col * CELL;
      if (col === 0 || col === COLS - 1) {
        const line = new Graphics();
        line.moveTo(x, ORIGIN_Y);
        line.lineTo(x, ORIGIN_Y + (ROWS - 1) * CELL);
        line.stroke({ width: 1.5, color: BOARD_STROKE });
        this.container.addChild(line);
      } else {
        // Top half (rows 0..4 from the rendered top, which is black side).
        const top = new Graphics();
        top.moveTo(x, ORIGIN_Y);
        top.lineTo(x, ORIGIN_Y + 4 * CELL);
        top.stroke({ width: 1.5, color: BOARD_STROKE });
        this.container.addChild(top);
        // Bottom half (rows 5..9).
        const bot = new Graphics();
        bot.moveTo(x, ORIGIN_Y + 5 * CELL);
        bot.lineTo(x, ORIGIN_Y + (ROWS - 1) * CELL);
        bot.stroke({ width: 1.5, color: BOARD_STROKE });
        this.container.addChild(bot);
      }
    }

    // Horizontal lines: 10 ranks, full width.
    for (let row = 0; row < ROWS; row++) {
      const y = ORIGIN_Y + row * CELL;
      const line = new Graphics();
      line.moveTo(ORIGIN_X, y);
      line.lineTo(ORIGIN_X + (COLS - 1) * CELL, y);
      line.stroke({ width: 1.5, color: BOARD_STROKE });
      this.container.addChild(line);
    }

    // Palace diagonals. Red palace = cols 3..5, rows 0..2.
    // Black palace = cols 3..5, rows 7..9. The rendered Y axis matches
    // model rows (we render row 9 at the bottom, see toCanvas).
    this.drawPalaceX(3, 0, 5, 2);
    this.drawPalaceX(3, 7, 5, 9);

    // River label area (between rank 4 and rank 5). Just a hint.
    const river = new Text({
      text: '楚河            漢界',
      style: { fontSize: 14, fill: BOARD_STROKE, fontStyle: 'italic' },
    });
    river.x = ORIGIN_X + ((COLS - 1) * CELL) / 2 - river.width / 2;
    river.y = ORIGIN_Y + 4 * CELL + (CELL - river.height) / 2;
    this.container.addChild(river);
  }

  private drawPalaceX(c1: number, r1: number, c2: number, r2: number): void {
    const a = this.toCanvas(r1, c1);
    const b = this.toCanvas(r2, c2);
    const c = this.toCanvas(r1, c2);
    const d = this.toCanvas(r2, c1);
    const g = new Graphics();
    g.moveTo(a.x, a.y);
    g.lineTo(b.x, b.y);
    g.moveTo(c.x, c.y);
    g.lineTo(d.x, d.y);
    g.stroke({ width: 1.5, color: BOARD_STROKE });
    this.container.addChild(g);
  }

  private drawHighlights(): void {
    if (this.selected) {
      const { x, y } = this.toCanvas(this.selected.row, this.selected.col);
      const ring = new Graphics();
      ring.circle(x, y, CELL * 0.46);
      ring.stroke({ width: 3, color: HIGHLIGHT_SELECT });
      this.container.addChild(ring);
    }
    for (const dest of this.legalDests) {
      const { x, y } = this.toCanvas(dest.row, dest.col);
      const dot = new Graphics();
      // Bigger ring for capture squares (target has a piece), small dot
      // for empty squares.
      const occupied = this.board[dest.row * COLS + dest.col] !== null;
      if (occupied) {
        dot.circle(x, y, CELL * 0.42);
        dot.stroke({ width: 3, color: HIGHLIGHT_DEST });
      } else {
        dot.circle(x, y, CELL * 0.18);
        dot.fill(HIGHLIGHT_DEST);
      }
      this.container.addChild(dot);
    }
  }

  private drawPieces(): void {
    const me = this.ctx.myIndex;
    const isMyTurn =
      this.gameState.status === 'playing' && me >= 0 && this.moveCount % 2 === me;

    for (let row = 0; row < ROWS; row++) {
      for (let col = 0; col < COLS; col++) {
        const piece = this.board[row * COLS + col];
        if (!piece) continue;
        const { x, y } = this.toCanvas(row, col);
        const color = piece.side === 0 ? RED_COLOR : BLACK_COLOR;

        const disc = new Graphics();
        disc.circle(x, y, CELL * 0.42);
        disc.fill(0xfff8e1);
        disc.stroke({ width: 2.5, color });
        this.container.addChild(disc);

        const glyph = piece.side === 0 ? RED_GLYPH[piece.kind] : BLACK_GLYPH[piece.kind];
        const text = new Text({
          text: glyph,
          style: {
            fontSize: 22,
            fill: color,
            fontWeight: 'bold',
            fontFamily: 'serif',
          },
        });
        text.x = x - text.width / 2;
        text.y = y - text.height / 2;
        this.container.addChild(text);

        if (isMyTurn && piece.side === me) {
          disc.interactive = true;
          disc.cursor = 'pointer';
          disc.on('pointerdown', () => this.selectPiece(row, col));
        }
      }
    }

    // Empty-cell click handlers for moving the selected piece.
    if (this.selected) {
      for (const dest of this.legalDests) {
        const { x, y } = this.toCanvas(dest.row, dest.col);
        const target = new Graphics();
        target.circle(x, y, CELL * 0.46);
        target.fill({ color: 0x000000, alpha: 0.001 }); // invisible hit area
        target.interactive = true;
        target.cursor = 'pointer';
        target.on('pointerdown', () => this.commitMove(dest.row, dest.col));
        this.container.addChild(target);
      }
    }
  }

  /** Map model (row, col) to canvas coordinates. Player 0 is red and sits
   * at the bottom: row 0 maps to the bottom of the canvas. */
  private toCanvas(row: number, col: number): { x: number; y: number } {
    const x = ORIGIN_X + col * CELL;
    // Flip vertically so red (row 0) is at the bottom for player 0.
    // For player 1 (black), we keep the same view: black sits at top,
    // which matches their natural orientation. Both players see red on
    // the bottom — a common convention for shared-display xiangqi.
    const y = ORIGIN_Y + (ROWS - 1 - row) * CELL;
    return { x, y };
  }

  private selectPiece(row: number, col: number): void {
    if (!this.wsConnected || this.gameState.status !== 'playing') return;
    const me = this.ctx.myIndex;
    if (me < 0 || this.moveCount % 2 !== me) return;
    const piece = this.board[row * COLS + col];
    if (!piece || piece.side !== me) return;

    if (this.selected && this.selected.row === row && this.selected.col === col) {
      this.selected = null;
      this.legalDests = [];
    } else {
      this.selected = { row, col };
      this.legalDests = computeLegalDests(this.board, row, col, me as Side);
    }
    this.refresh();
  }

  private commitMove(toRow: number, toCol: number): void {
    if (!this.selected) return;
    const me = this.ctx.myIndex;
    if (me < 0 || this.moveCount % 2 !== me) return;
    const from = squareToStr(this.selected.row, this.selected.col);
    const to = squareToStr(toRow, toCol);
    this.sendMove(`${from} ${to}`);
    this.selected = null;
    this.legalDests = [];
  }
}

// -------------------- helpers (no imports outside pixi) --------------------

function initialBoard(): (Piece | null)[] {
  const b: (Piece | null)[] = Array(COLS * ROWS).fill(null);
  const back: Kind[] = ['R', 'H', 'E', 'A', 'K', 'A', 'E', 'H', 'R'];
  for (let col = 0; col < COLS; col++) {
    b[0 * COLS + col] = { side: 0, kind: back[col] };
    b[9 * COLS + col] = { side: 1, kind: back[col] };
  }
  for (const col of [1, 7]) {
    b[2 * COLS + col] = { side: 0, kind: 'C' };
    b[7 * COLS + col] = { side: 1, kind: 'C' };
  }
  for (const col of [0, 2, 4, 6, 8]) {
    b[3 * COLS + col] = { side: 0, kind: 'P' };
    b[6 * COLS + col] = { side: 1, kind: 'P' };
  }
  return b;
}

function parseSquare(s: string): { row: number; col: number } | null {
  if (s.length < 2 || s.length > 3) return null;
  const file = s.charCodeAt(0);
  if (file < 'a'.charCodeAt(0) || file > 'i'.charCodeAt(0)) return null;
  const col = file - 'a'.charCodeAt(0);
  const row = parseInt(s.slice(1), 10);
  if (Number.isNaN(row) || row < 0 || row >= ROWS) return null;
  return { row, col };
}

function squareToStr(row: number, col: number): string {
  return String.fromCharCode('a'.charCodeAt(0) + col) + row;
}

// -------------------- legal-move generator (frontend mirror) --------------------
// Mirrors judge/src/games/xiangqi_logic.rs so the UI can highlight
// legal targets without round-tripping the server. The judge re-validates
// every MOVE, so this is a UX hint only.

function inBoard(r: number, c: number): boolean {
  return r >= 0 && r < ROWS && c >= 0 && c < COLS;
}

function inPalace(side: Side, r: number, c: number): boolean {
  if (c < 3 || c > 5) return false;
  return side === 0 ? r >= 0 && r <= 2 : r >= 7 && r <= 9;
}

function crossedRiver(side: Side, r: number): boolean {
  return side === 0 ? r >= 5 : r <= 4;
}

function onOwnSide(side: Side, r: number): boolean {
  return side === 0 ? r <= 4 : r >= 5;
}

function pseudoMoves(
  board: (Piece | null)[],
  fr: number,
  fc: number,
): { row: number; col: number }[] {
  const piece = board[fr * COLS + fc];
  if (!piece) return [];
  const out: { row: number; col: number }[] = [];

  const push = (nr: number, nc: number) => {
    if (!inBoard(nr, nc)) return;
    const t = board[nr * COLS + nc];
    if (t && t.side === piece.side) return;
    out.push({ row: nr, col: nc });
  };

  switch (piece.kind) {
    case 'K': {
      for (const [dr, dc] of [
        [1, 0],
        [-1, 0],
        [0, 1],
        [0, -1],
      ]) {
        const nr = fr + dr;
        const nc = fc + dc;
        if (inPalace(piece.side, nr, nc)) push(nr, nc);
      }
      break;
    }
    case 'A': {
      for (const [dr, dc] of [
        [1, 1],
        [1, -1],
        [-1, 1],
        [-1, -1],
      ]) {
        const nr = fr + dr;
        const nc = fc + dc;
        if (inPalace(piece.side, nr, nc)) push(nr, nc);
      }
      break;
    }
    case 'E': {
      for (const [dr, dc] of [
        [2, 2],
        [2, -2],
        [-2, 2],
        [-2, -2],
      ]) {
        const nr = fr + dr;
        const nc = fc + dc;
        if (!inBoard(nr, nc)) continue;
        if (!onOwnSide(piece.side, nr)) continue;
        const mr = fr + dr / 2;
        const mc = fc + dc / 2;
        if (board[mr * COLS + mc]) continue;
        push(nr, nc);
      }
      break;
    }
    case 'H': {
      const jumps: [number, number, number, number][] = [
        [2, 1, 1, 0],
        [2, -1, 1, 0],
        [-2, 1, -1, 0],
        [-2, -1, -1, 0],
        [1, 2, 0, 1],
        [-1, 2, 0, 1],
        [1, -2, 0, -1],
        [-1, -2, 0, -1],
      ];
      for (const [dr, dc, lr, lc] of jumps) {
        const nr = fr + dr;
        const nc = fc + dc;
        if (!inBoard(nr, nc)) continue;
        if (board[(fr + lr) * COLS + (fc + lc)]) continue;
        push(nr, nc);
      }
      break;
    }
    case 'R': {
      for (const [dr, dc] of [
        [1, 0],
        [-1, 0],
        [0, 1],
        [0, -1],
      ]) {
        let nr = fr + dr;
        let nc = fc + dc;
        let guard = 0;
        while (inBoard(nr, nc) && guard++ < ROWS + COLS) {
          const t = board[nr * COLS + nc];
          if (!t) {
            out.push({ row: nr, col: nc });
          } else {
            if (t.side !== piece.side) out.push({ row: nr, col: nc });
            break;
          }
          nr += dr;
          nc += dc;
        }
      }
      break;
    }
    case 'C': {
      for (const [dr, dc] of [
        [1, 0],
        [-1, 0],
        [0, 1],
        [0, -1],
      ]) {
        let nr = fr + dr;
        let nc = fc + dc;
        let guard = 0;
        // Phase 1: empty slides (non-capture).
        while (inBoard(nr, nc) && !board[nr * COLS + nc] && guard++ < ROWS + COLS) {
          out.push({ row: nr, col: nc });
          nr += dr;
          nc += dc;
        }
        if (!inBoard(nr, nc)) continue;
        // Phase 2: jump exactly one screen, capture next piece.
        nr += dr;
        nc += dc;
        guard = 0;
        while (inBoard(nr, nc) && guard++ < ROWS + COLS) {
          const t = board[nr * COLS + nc];
          if (t) {
            if (t.side !== piece.side) out.push({ row: nr, col: nc });
            break;
          }
          nr += dr;
          nc += dc;
        }
      }
      break;
    }
    case 'P': {
      const forward = piece.side === 0 ? 1 : -1;
      push(fr + forward, fc);
      if (crossedRiver(piece.side, fr)) {
        push(fr, fc + 1);
        push(fr, fc - 1);
      }
      break;
    }
  }

  return out;
}

function findGeneral(board: (Piece | null)[], side: Side): { row: number; col: number } | null {
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const p = board[r * COLS + c];
      if (p && p.side === side && p.kind === 'K') return { row: r, col: c };
    }
  }
  return null;
}

function generalsFace(board: (Piece | null)[]): boolean {
  const g0 = findGeneral(board, 0);
  const g1 = findGeneral(board, 1);
  if (!g0 || !g1) return false;
  if (g0.col !== g1.col) return false;
  const lo = Math.min(g0.row, g1.row);
  const hi = Math.max(g0.row, g1.row);
  for (let r = lo + 1; r < hi; r++) {
    if (board[r * COLS + g0.col]) return false;
  }
  return true;
}

function inCheck(board: (Piece | null)[], side: Side): boolean {
  const k = findGeneral(board, side);
  if (!k) return false;
  const opp = (1 - side) as Side;
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const p = board[r * COLS + c];
      if (!p || p.side !== opp) continue;
      for (const m of pseudoMoves(board, r, c)) {
        if (m.row === k.row && m.col === k.col) return true;
      }
    }
  }
  return false;
}

function computeLegalDests(
  board: (Piece | null)[],
  fr: number,
  fc: number,
  side: Side,
): { row: number; col: number }[] {
  const out: { row: number; col: number }[] = [];
  for (const m of pseudoMoves(board, fr, fc)) {
    const next = board.slice();
    next[m.row * COLS + m.col] = next[fr * COLS + fc];
    next[fr * COLS + fc] = null;
    if (inCheck(next, side)) continue;
    if (generalsFace(next)) continue;
    out.push(m);
  }
  return out;
}
// COLORS reference kept for parity with sibling games (lint-friendly).
void COLORS;
