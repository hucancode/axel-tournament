import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

/** Spec: judge/protocols/chess.md. Player 0 = white, player 1 = black. */

type Color = 'w' | 'b';
type PieceType = 'P' | 'N' | 'B' | 'R' | 'Q' | 'K';
interface Piece {
  color: Color;
  type: PieceType;
}

type DrawReason = 'STALEMATE' | 'FIFTY_MOVE' | 'THREEFOLD' | 'INSUFFICIENT';

const SQUARE = 50;
const BOARD_PX = SQUARE * 8;

const LIGHT_SQUARE = 0xf0d9b5;
const DARK_SQUARE = 0xb58863;
const SELECT_HIGHLIGHT = 0x6ab04c;
const LEGAL_HIGHLIGHT = 0xfde047;
const LAST_MOVE_HIGHLIGHT = 0xfff59d;

const GLYPHS: Record<Color, Record<PieceType, string>> = {
  w: { K: '♔', Q: '♕', R: '♖', B: '♗', N: '♘', P: '♙' },
  b: { K: '♚', Q: '♛', R: '♜', B: '♝', N: '♞', P: '♟' },
};

export class ChessGame extends BasePixiGame {
  private board: (Piece | null)[] = startingBoard();
  private selected: number | null = null;
  // Highlights mirror the legal move set the server would accept. We
  // recompute them client-side from the local board so click-to-move
  // feels responsive — illegal clicks still get rejected by the server.
  private legalDestinations: Set<number> = new Set();
  private lastMove: { from: number; to: number } | null = null;
  private moveCount = 0;
  private inCheck: number | null = null;
  private outcome: 'win' | 'lose' | 'draw' | null = null;
  private drawReason: DrawReason | null = null;
  // Track castling rights and en passant target so the renderer's
  // local legal-move generator accepts the same moves the server does.
  private castle = { wk: true, wq: true, bk: true, bq: true };
  private enPassant: number | null = null;
  private pendingPromotion: { from: number; to: number } | null = null;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;

    switch (kind) {
      case 'GAME_STARTED':
        this.board = startingBoard();
        this.selected = null;
        this.legalDestinations.clear();
        this.lastMove = null;
        this.moveCount = 0;
        this.inCheck = null;
        this.outcome = null;
        this.drawReason = null;
        this.castle = { wk: true, wq: true, bk: true, bq: true };
        this.enPassant = null;
        this.pendingPromotion = null;
        this.gameState.status = 'playing';
        this.refresh();
        break;
      case 'MOVE': {
        const parts = payload.split(' ');
        if (parts.length !== 4) return;
        const moverIdx = ctx.players.indexOf(parts[0]);
        const from = parseSquare(parts[1]);
        const to = parseSquare(parts[2]);
        const flag = parts[3];
        if (moverIdx < 0 || from < 0 || to < 0) return;
        this.applyMove(from, to, flag);
        this.lastMove = { from, to };
        this.moveCount += 1;
        this.selected = null;
        this.legalDestinations.clear();
        this.inCheck = null;
        this.refresh();
        break;
      }
      case 'CHECK': {
        const idx = parseInt(payload, 10);
        if (!isNaN(idx)) this.inCheck = idx;
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
      case 'DRAW': {
        this.gameState.status = 'finished';
        this.outcome = 'draw';
        this.drawReason = (payload.trim() as DrawReason) || null;
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
      if (this.outcome === 'win') {
        return { text: 'Checkmate – You Win', tone: 'win' };
      }
      if (this.outcome === 'lose') {
        return { text: 'Checkmate – You Lose', tone: 'lose' };
      }
      return { text: 'Draw', tone: 'draw', detail: this.drawDetail() };
    }
    const me = this.ctx.myIndex;
    const colorLabel = me === 0 ? 'White' : 'Black';
    const isMyTurn = me >= 0 && this.moveCount % 2 === me;
    if (isMyTurn) {
      const detail = this.inCheck === me ? `You are in check · Playing ${colorLabel}` : `Playing ${colorLabel}`;
      return { text: 'Your turn', tone: 'turn', detail };
    }
    const detail = this.inCheck === me ? `You are in check · You are ${colorLabel}` : `You are ${colorLabel}`;
    return { text: "Opponent's turn", tone: 'wait', detail };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished' || !this.outcome) return null;
    const colorLabel = this.ctx.myIndex === 0 ? 'White' : 'Black';
    const title =
      this.outcome === 'win' ? 'Checkmate'
        : this.outcome === 'lose' ? 'Defeat'
        : 'Draw';
    const details = [`You played ${colorLabel}`, `${this.moveCount} moves played`];
    if (this.outcome === 'draw' && this.drawReason) {
      details.push(`Reason: ${formatDrawReason(this.drawReason)}`);
    }
    return { outcome: this.outcome, title, details };
  }

  protected render(): void {
    this.container.removeChildren();

    const me = this.ctx.myIndex;
    const flipped = me === 1; // black at the bottom for black player
    const isMyTurn =
      this.gameState.status === 'playing' && me >= 0 && this.moveCount % 2 === me;

    for (let sq = 0; sq < 64; sq++) {
      const file = sq % 8;
      const rank = Math.floor(sq / 8);
      const screenFile = flipped ? 7 - file : file;
      const screenRank = flipped ? rank : 7 - rank;
      const x = screenFile * SQUARE;
      const y = screenRank * SQUARE;
      const isLight = (file + rank) % 2 === 1;
      const baseColor = isLight ? LIGHT_SQUARE : DARK_SQUARE;

      const cell = new Graphics();
      cell.rect(x, y, SQUARE, SQUARE);
      cell.fill(baseColor);
      this.container.addChild(cell);

      // Last move tint underneath highlights so highlights win.
      if (this.lastMove && (sq === this.lastMove.from || sq === this.lastMove.to)) {
        const tint = new Graphics();
        tint.rect(x, y, SQUARE, SQUARE);
        tint.fill({ color: LAST_MOVE_HIGHLIGHT, alpha: 0.5 });
        this.container.addChild(tint);
      }

      if (this.selected === sq) {
        const sel = new Graphics();
        sel.rect(x, y, SQUARE, SQUARE);
        sel.fill({ color: SELECT_HIGHLIGHT, alpha: 0.5 });
        this.container.addChild(sel);
      } else if (this.legalDestinations.has(sq)) {
        const dot = new Graphics();
        dot.circle(x + SQUARE / 2, y + SQUARE / 2, 8);
        dot.fill({ color: LEGAL_HIGHLIGHT, alpha: 0.85 });
        this.container.addChild(dot);
      }

      // Coordinate labels on the outer files / ranks.
      if (screenFile === 0) {
        const label = new Text({
          text: String(rank + 1),
          style: { fontSize: 10, fill: isLight ? DARK_SQUARE : LIGHT_SQUARE, fontWeight: 'bold' },
        });
        label.x = x + 2;
        label.y = y + 2;
        this.container.addChild(label);
      }
      if (screenRank === 7) {
        const label = new Text({
          text: String.fromCharCode('a'.charCodeAt(0) + file),
          style: { fontSize: 10, fill: isLight ? DARK_SQUARE : LIGHT_SQUARE, fontWeight: 'bold' },
        });
        label.x = x + SQUARE - 10;
        label.y = y + SQUARE - 14;
        this.container.addChild(label);
      }

      const piece = this.board[sq];
      if (piece) {
        const text = new Text({
          text: GLYPHS[piece.color][piece.type],
          style: {
            fontSize: 38,
            fill: piece.color === 'w' ? COLORS.WHITE : COLORS.BLACK,
            fontWeight: 'bold',
            stroke: { color: piece.color === 'w' ? COLORS.BLACK : COLORS.WHITE, width: 2 },
          },
        });
        text.x = x + SQUARE / 2 - text.width / 2;
        text.y = y + SQUARE / 2 - text.height / 2;
        this.container.addChild(text);
      }

      if (this.gameState.status === 'playing' && isMyTurn) {
        cell.interactive = true;
        cell.cursor = 'pointer';
        cell.on('pointerdown', () => this.onSquareClick(sq));
      }
    }
  }

  private onSquareClick(sq: number): void {
    const me = this.ctx.myIndex;
    if (me < 0 || this.gameState.status !== 'playing') return;
    if (this.moveCount % 2 !== me) return;

    const myColor: Color = me === 0 ? 'w' : 'b';
    const piece = this.board[sq];

    if (this.selected === null) {
      if (piece && piece.color === myColor) {
        this.selected = sq;
        this.legalDestinations = new Set(this.computeLegalDestinations(sq));
        this.refresh();
      }
      return;
    }
    if (this.selected === sq) {
      this.selected = null;
      this.legalDestinations.clear();
      this.refresh();
      return;
    }
    if (piece && piece.color === myColor) {
      // Re-select another own piece.
      this.selected = sq;
      this.legalDestinations = new Set(this.computeLegalDestinations(sq));
      this.refresh();
      return;
    }
    if (this.legalDestinations.has(sq)) {
      const from = this.selected;
      const to = sq;
      if (this.isPromotion(from, to)) {
        this.pendingPromotion = { from, to };
        // Always promote to queen client-side; the spec lets the server
        // accept any q|r|b|n. Simpler than a popup, matches the bot.
        this.sendChessMove(from, to, 'q');
      } else {
        this.sendChessMove(from, to, null);
      }
    }
  }

  private sendChessMove(from: number, to: number, promo: string | null): void {
    if (!this.wsConnected) return;
    const payload = promo
      ? `${squareStr(from)} ${squareStr(to)} ${promo}`
      : `${squareStr(from)} ${squareStr(to)}`;
    this.sendMove(payload);
  }

  private isPromotion(from: number, to: number): boolean {
    const piece = this.board[from];
    if (!piece || piece.type !== 'P') return false;
    const toRank = Math.floor(to / 8);
    return (piece.color === 'w' && toRank === 7) || (piece.color === 'b' && toRank === 0);
  }

  /** Apply a confirmed server MOVE event to the local board. */
  private applyMove(from: number, to: number, flag: string): void {
    const piece = this.board[from];
    if (!piece) return;

    this.board[from] = null;
    this.board[to] = piece;

    if (flag === 'K') {
      const rank = Math.floor(to / 8);
      const rook = this.board[rank * 8 + 7];
      this.board[rank * 8 + 7] = null;
      this.board[rank * 8 + 5] = rook;
    } else if (flag === 'Q') {
      const rank = Math.floor(to / 8);
      const rook = this.board[rank * 8];
      this.board[rank * 8] = null;
      this.board[rank * 8 + 3] = rook;
    } else if (flag === 'E') {
      const capRank = piece.color === 'w' ? Math.floor(to / 8) - 1 : Math.floor(to / 8) + 1;
      const capFile = to % 8;
      this.board[capRank * 8 + capFile] = null;
    } else if (flag === 'q' || flag === 'r' || flag === 'b' || flag === 'n') {
      const promoMap: Record<string, PieceType> = { q: 'Q', r: 'R', b: 'B', n: 'N' };
      this.board[to] = { color: piece.color, type: promoMap[flag] };
    }

    // Update castling rights mirror.
    if (piece.type === 'K') {
      if (piece.color === 'w') { this.castle.wk = false; this.castle.wq = false; }
      else { this.castle.bk = false; this.castle.bq = false; }
    }
    for (const s of [from, to]) {
      if (s === 0) this.castle.wq = false;
      if (s === 7) this.castle.wk = false;
      if (s === 56) this.castle.bq = false;
      if (s === 63) this.castle.bk = false;
    }

    // En-passant target is set only after a pawn double-step.
    const fromRank = Math.floor(from / 8);
    const toRank = Math.floor(to / 8);
    if (piece.type === 'P' && Math.abs(toRank - fromRank) === 2) {
      this.enPassant = (fromRank + toRank) / 2 * 8 + (from % 8);
    } else {
      this.enPassant = null;
    }

    this.pendingPromotion = null;
  }

  /**
   * Compute legal destinations for the piece on `from`. Mirrors the
   * server's generator: pseudo-legal moves filtered by "doesn't leave
   * own king in check". Castling and en passant included.
   *
   * Bounded loops only (max 7 ray steps; 64 squares for king-square
   * scan), so worst-case work per click is small.
   */
  private computeLegalDestinations(from: number): number[] {
    const piece = this.board[from];
    if (!piece) return [];
    const pseudo = this.pseudoMoves(from, piece, this.board, this.castle, this.enPassant);
    const out: number[] = [];
    for (const m of pseudo) {
      const next = simulate(this.board, from, m.to, m.flag, piece);
      if (!isKingAttacked(next, piece.color)) {
        out.push(m.to);
      }
    }
    return out;
  }

  private pseudoMoves(
    from: number,
    piece: Piece,
    board: (Piece | null)[],
    castle: { wk: boolean; wq: boolean; bk: boolean; bq: boolean },
    ep: number | null,
  ): { to: number; flag: string }[] {
    const moves: { to: number; flag: string }[] = [];
    const file = from % 8;
    const rank = Math.floor(from / 8);
    const sq = (f: number, r: number) => r * 8 + f;
    const inBounds = (f: number, r: number) => f >= 0 && f < 8 && r >= 0 && r < 8;

    const slide = (dirs: [number, number][]) => {
      for (const [df, dr] of dirs) {
        for (let step = 1; step < 8; step++) {
          const f = file + df * step;
          const r = rank + dr * step;
          if (!inBounds(f, r)) break;
          const target = board[sq(f, r)];
          if (!target) {
            moves.push({ to: sq(f, r), flag: '-' });
          } else {
            if (target.color !== piece.color) moves.push({ to: sq(f, r), flag: '-' });
            break;
          }
        }
      }
    };

    if (piece.type === 'P') {
      const dir = piece.color === 'w' ? 1 : -1;
      const startRank = piece.color === 'w' ? 1 : 6;
      const f1 = sq(file, rank + dir);
      if (inBounds(file, rank + dir) && !board[f1]) {
        moves.push({ to: f1, flag: '-' });
        if (rank === startRank) {
          const f2 = sq(file, rank + dir * 2);
          if (!board[f2]) moves.push({ to: f2, flag: '-' });
        }
      }
      for (const df of [-1, 1]) {
        if (!inBounds(file + df, rank + dir)) continue;
        const t = sq(file + df, rank + dir);
        const target = board[t];
        if (target && target.color !== piece.color) moves.push({ to: t, flag: '-' });
        if (ep !== null && t === ep) moves.push({ to: t, flag: 'E' });
      }
    } else if (piece.type === 'N') {
      const deltas: [number, number][] = [
        [1, 2], [2, 1], [2, -1], [1, -2],
        [-1, -2], [-2, -1], [-2, 1], [-1, 2],
      ];
      for (const [df, dr] of deltas) {
        if (!inBounds(file + df, rank + dr)) continue;
        const t = sq(file + df, rank + dr);
        const target = board[t];
        if (!target || target.color !== piece.color) moves.push({ to: t, flag: '-' });
      }
    } else if (piece.type === 'B') {
      slide([[1, 1], [1, -1], [-1, 1], [-1, -1]]);
    } else if (piece.type === 'R') {
      slide([[1, 0], [-1, 0], [0, 1], [0, -1]]);
    } else if (piece.type === 'Q') {
      slide([[1, 1], [1, -1], [-1, 1], [-1, -1], [1, 0], [-1, 0], [0, 1], [0, -1]]);
    } else if (piece.type === 'K') {
      const deltas: [number, number][] = [
        [1, 0], [1, 1], [0, 1], [-1, 1],
        [-1, 0], [-1, -1], [0, -1], [1, -1],
      ];
      for (const [df, dr] of deltas) {
        if (!inBounds(file + df, rank + dr)) continue;
        const t = sq(file + df, rank + dr);
        const target = board[t];
        if (!target || target.color !== piece.color) moves.push({ to: t, flag: '-' });
      }
      // Castling. Only emitted from the king's starting square; legal
      // filter strips it if the king passes through attacked squares.
      const homeRank = piece.color === 'w' ? 0 : 7;
      if (file === 4 && rank === homeRank && !isKingAttacked(board, piece.color)) {
        const opp: Color = piece.color === 'w' ? 'b' : 'w';
        const kRight = piece.color === 'w' ? castle.wk : castle.bk;
        const qRight = piece.color === 'w' ? castle.wq : castle.bq;
        if (kRight
          && !board[sq(5, homeRank)]
          && !board[sq(6, homeRank)]
          && !isSquareAttacked(board, sq(5, homeRank), opp)
          && !isSquareAttacked(board, sq(6, homeRank), opp)) {
          moves.push({ to: sq(6, homeRank), flag: 'K' });
        }
        if (qRight
          && !board[sq(1, homeRank)]
          && !board[sq(2, homeRank)]
          && !board[sq(3, homeRank)]
          && !isSquareAttacked(board, sq(3, homeRank), opp)
          && !isSquareAttacked(board, sq(2, homeRank), opp)) {
          moves.push({ to: sq(2, homeRank), flag: 'Q' });
        }
      }
    }
    return moves;
  }

  private drawDetail(): string {
    if (!this.drawReason) return '';
    return formatDrawReason(this.drawReason);
  }
}

function startingBoard(): (Piece | null)[] {
  const b: (Piece | null)[] = Array(64).fill(null);
  const back: PieceType[] = ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'];
  for (let f = 0; f < 8; f++) {
    b[f] = { color: 'w', type: back[f] };
    b[8 + f] = { color: 'w', type: 'P' };
    b[48 + f] = { color: 'b', type: 'P' };
    b[56 + f] = { color: 'b', type: back[f] };
  }
  return b;
}

function parseSquare(s: string): number {
  if (s.length !== 2) return -1;
  const f = s.charCodeAt(0) - 'a'.charCodeAt(0);
  const r = s.charCodeAt(1) - '1'.charCodeAt(0);
  if (f < 0 || f > 7 || r < 0 || r > 7) return -1;
  return r * 8 + f;
}

function squareStr(sq: number): string {
  const f = sq % 8;
  const r = Math.floor(sq / 8);
  return String.fromCharCode('a'.charCodeAt(0) + f) + String.fromCharCode('1'.charCodeAt(0) + r);
}

function formatDrawReason(reason: DrawReason): string {
  switch (reason) {
    case 'STALEMATE': return 'Stalemate';
    case 'FIFTY_MOVE': return '50-move rule';
    case 'THREEFOLD': return 'Threefold repetition';
    case 'INSUFFICIENT': return 'Insufficient material';
  }
}

/** Apply a move to a copy of the board (no rights / ep tracking — just
 *  the squares — sufficient for the in-check filter). */
function simulate(
  board: (Piece | null)[],
  from: number,
  to: number,
  flag: string,
  piece: Piece,
): (Piece | null)[] {
  const b = board.slice();
  b[from] = null;
  b[to] = piece;
  if (flag === 'K') {
    const rank = Math.floor(to / 8);
    const rook = b[rank * 8 + 7];
    b[rank * 8 + 7] = null;
    b[rank * 8 + 5] = rook;
  } else if (flag === 'Q') {
    const rank = Math.floor(to / 8);
    const rook = b[rank * 8];
    b[rank * 8] = null;
    b[rank * 8 + 3] = rook;
  } else if (flag === 'E') {
    const capRank = piece.color === 'w' ? Math.floor(to / 8) - 1 : Math.floor(to / 8) + 1;
    const capFile = to % 8;
    b[capRank * 8 + capFile] = null;
  }
  return b;
}

function isSquareAttacked(board: (Piece | null)[], target: number, by: Color): boolean {
  const tf = target % 8;
  const tr = Math.floor(target / 8);
  // Pawn attacks (by attacker direction).
  const pawnDir = by === 'w' ? 1 : -1;
  for (const df of [-1, 1]) {
    const f = tf - df;
    const r = tr - pawnDir;
    if (f >= 0 && f < 8 && r >= 0 && r < 8) {
      const p = board[r * 8 + f];
      if (p && p.color === by && p.type === 'P') return true;
    }
  }
  // Knight.
  const knight: [number, number][] = [
    [1, 2], [2, 1], [2, -1], [1, -2],
    [-1, -2], [-2, -1], [-2, 1], [-1, 2],
  ];
  for (const [df, dr] of knight) {
    const f = tf + df, r = tr + dr;
    if (f < 0 || f > 7 || r < 0 || r > 7) continue;
    const p = board[r * 8 + f];
    if (p && p.color === by && p.type === 'N') return true;
  }
  // King.
  for (let df = -1; df <= 1; df++) {
    for (let dr = -1; dr <= 1; dr++) {
      if (df === 0 && dr === 0) continue;
      const f = tf + df, r = tr + dr;
      if (f < 0 || f > 7 || r < 0 || r > 7) continue;
      const p = board[r * 8 + f];
      if (p && p.color === by && p.type === 'K') return true;
    }
  }
  // Bishops + queens (diagonal rays).
  for (const [df, dr] of [[1, 1], [1, -1], [-1, 1], [-1, -1]] as [number, number][]) {
    for (let step = 1; step < 8; step++) {
      const f = tf + df * step, r = tr + dr * step;
      if (f < 0 || f > 7 || r < 0 || r > 7) break;
      const p = board[r * 8 + f];
      if (!p) continue;
      if (p.color === by && (p.type === 'B' || p.type === 'Q')) return true;
      break;
    }
  }
  // Rooks + queens (orthogonal rays).
  for (const [df, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as [number, number][]) {
    for (let step = 1; step < 8; step++) {
      const f = tf + df * step, r = tr + dr * step;
      if (f < 0 || f > 7 || r < 0 || r > 7) break;
      const p = board[r * 8 + f];
      if (!p) continue;
      if (p.color === by && (p.type === 'R' || p.type === 'Q')) return true;
      break;
    }
  }
  return false;
}

function isKingAttacked(board: (Piece | null)[], color: Color): boolean {
  for (let i = 0; i < 64; i++) {
    const p = board[i];
    if (p && p.color === color && p.type === 'K') {
      return isSquareAttacked(board, i, color === 'w' ? 'b' : 'w');
    }
  }
  return false;
}
