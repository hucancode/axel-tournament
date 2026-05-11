import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

/**
 * Heads-up no-limit Texas Hold'em.
 *
 * Spec: judge/protocols/poker.md.
 *
 * Layout:
 *   - top:    opponent stack, opponent bet this street, opponent hole cards (face-down or revealed at showdown)
 *   - middle: community cards (board) + pot
 *   - bottom: your stack, your bet this street, your hole cards (face-up), action buttons
 */
type Street = 'preflop' | 'flop' | 'turn' | 'river' | 'settled';

interface Card {
  rank: string;
  suit: string;
}

const CARD_W = 44;
const CARD_H = 62;
const CANVAS_W = 400;
const CANVAS_H = 400;

const ACTION_VERBS = ['FOLD', 'CHECK', 'CALL', 'BET', 'RAISE', 'ALLIN'] as const;

function parseCard(s: string): Card | null {
  if (s.length !== 2) return null;
  return { rank: s[0], suit: s[1] };
}

function suitGlyph(suit: string): string {
  switch (suit) {
    case 'h':
      return '♥';
    case 'd':
      return '♦';
    case 'c':
      return '♣';
    case 's':
      return '♠';
    default:
      return '?';
  }
}

function isRedSuit(suit: string): boolean {
  return suit === 'h' || suit === 'd';
}

export class PokerGame extends BasePixiGame {
  private numHands = 0;
  private startingStack = 0;
  private smallBlind = 0;
  private handNo = 0;
  private dealer = 0;
  private street: Street = 'settled';
  private hole: [Card | null, Card | null] = [null, null];
  private oppHole: [Card | null, Card | null] = [null, null];
  private board: Card[] = [];
  private stacks: [number, number] = [0, 0];
  private betInStreet: [number, number] = [0, 0];
  private committed: [number, number] = [0, 0];
  private folded: [boolean, boolean] = [false, false];
  private allIn: [boolean, boolean] = [false, false];
  private toAct = 0;
  private lastRaiseSize = 0;
  private potSnapshot = 0;
  private outcome: 'win' | 'lose' | 'draw' | null = null;
  private finalStacks: [number, number] = [0, 0];
  private revealOpponent = false;
  /** Pending custom bet/raise input for the action buttons. */
  private betInput = 0;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;
    const me = ctx.myIndex;

    switch (kind) {
      case 'GAME_STARTED': {
        const p = payload.split(' ');
        this.numHands = parseInt(p[0] ?? '', 10) || 10;
        this.startingStack = parseInt(p[1] ?? '', 10) || 1000;
        this.smallBlind = parseInt(p[2] ?? '', 10) || 10;
        this.stacks = [this.startingStack, this.startingStack];
        this.gameState.status = 'playing';
        this.outcome = null;
        this.betInput = this.smallBlind * 2;
        this.refresh();
        break;
      }
      case 'HAND_STARTED': {
        const p = payload.split(' ');
        this.handNo = parseInt(p[0] ?? '', 10) || 0;
        this.dealer = parseInt(p[1] ?? '', 10) || 0;
        const c0a = parseCard(p[2] ?? '');
        const c0b = parseCard(p[3] ?? '');
        const c1a = parseCard(p[4] ?? '');
        const c1b = parseCard(p[5] ?? '');
        this.hole = me === 0 ? [c0a, c0b] : [c1a, c1b];
        this.oppHole = me === 0 ? [c1a, c1b] : [c0a, c0b];
        this.board = [];
        this.betInStreet = [0, 0];
        this.committed = [0, 0];
        this.folded = [false, false];
        this.allIn = [false, false];
        this.lastRaiseSize = this.smallBlind * 2;
        this.toAct = this.dealer;
        this.street = 'preflop';
        this.revealOpponent = false;
        this.betInput = Math.max(this.smallBlind * 2, this.betInput);
        this.refresh();
        break;
      }
      case 'STREET': {
        const p = payload.split(' ');
        const name = p[0] as Street;
        this.street = name;
        for (let i = 1; i < p.length; i++) {
          const c = parseCard(p[i]);
          if (c) this.board.push(c);
        }
        this.betInStreet = [0, 0];
        this.lastRaiseSize = 0;
        this.toAct = 1 - this.dealer;
        this.refresh();
        break;
      }
      case 'ACTION': {
        const p = payload.split(' ');
        const pid = p[0];
        const verb = p[1];
        const amt = parseInt(p[2] ?? '0', 10) || 0;
        const idx = ctx.players.indexOf(pid);
        if (idx < 0) break;
        switch (verb) {
          case 'POST': {
            this.betInStreet[idx] = amt;
            this.committed[idx] = amt;
            this.stacks[idx] = Math.max(0, this.stacks[idx] - amt);
            if (this.stacks[idx] === 0) this.allIn[idx] = true;
            this.toAct = this.dealer;
            break;
          }
          case 'FOLD':
            this.folded[idx] = true;
            break;
          case 'CHECK':
            this.toAct = 1 - idx;
            break;
          case 'CALL': {
            this.stacks[idx] = Math.max(0, this.stacks[idx] - amt);
            this.committed[idx] += amt;
            this.betInStreet[idx] += amt;
            if (this.stacks[idx] === 0) this.allIn[idx] = true;
            this.toAct = 1 - idx;
            break;
          }
          case 'BET':
          case 'RAISE': {
            const prev = this.betInStreet[idx];
            const delta = Math.max(0, amt - prev);
            this.stacks[idx] = Math.max(0, this.stacks[idx] - delta);
            this.committed[idx] += delta;
            this.betInStreet[idx] = amt;
            if (this.stacks[idx] === 0) this.allIn[idx] = true;
            this.lastRaiseSize = Math.max(this.lastRaiseSize, delta);
            this.toAct = 1 - idx;
            break;
          }
          case 'ALLIN': {
            this.stacks[idx] = Math.max(0, this.stacks[idx] - amt);
            this.committed[idx] += amt;
            this.betInStreet[idx] += amt;
            this.allIn[idx] = true;
            this.toAct = 1 - idx;
            break;
          }
        }
        this.refresh();
        break;
      }
      case 'POT': {
        this.potSnapshot = parseInt(payload, 10) || 0;
        this.refresh();
        break;
      }
      case 'HAND_END': {
        const p = payload.split(' ');
        const winnerStr = p[0];
        const c0 = parseInt(p[1] ?? '0', 10) || 0;
        const c1 = parseInt(p[2] ?? '0', 10) || 0;
        const reason = p[3] ?? '';
        this.stacks = [c0, c1];
        this.committed = [0, 0];
        this.betInStreet = [0, 0];
        this.potSnapshot = 0;
        // Reveal opponent cards on showdown / split.
        this.revealOpponent = reason === 'SHOWDOWN' || reason === 'SPLIT';
        // Track local outcome detail purely for status text — final outcome
        // is determined at GAME_END.
        const _ = winnerStr;
        this.refresh();
        break;
      }
      case 'GAME_END': {
        const p = payload.split(' ');
        const c0 = parseInt(p[0] ?? '0', 10) || 0;
        const c1 = parseInt(p[1] ?? '0', 10) || 0;
        this.finalStacks = [c0, c1];
        const myStack = me === 0 ? c0 : c1;
        const oppStack = me === 0 ? c1 : c0;
        if (myStack > oppStack) this.outcome = 'win';
        else if (myStack < oppStack) this.outcome = 'lose';
        else this.outcome = 'draw';
        this.gameState.status = 'finished';
        this.refresh();
        break;
      }
      case 'WINNER': {
        // Already covered by GAME_END for stacks; ensure phase is finished.
        if (this.outcome === null) {
          const w = parseInt(payload, 10);
          this.outcome = w === me ? 'win' : 'lose';
        }
        this.gameState.status = 'finished';
        this.refresh();
        break;
      }
      case 'DRAW': {
        if (this.outcome === null) this.outcome = 'draw';
        this.gameState.status = 'finished';
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
      const detail = `Chips ${this.finalStacks[this.ctx.myIndex] ?? 0} – ${this.finalStacks[1 - this.ctx.myIndex] ?? 0}`;
      return { text: 'Final score', tone: 'idle', detail };
    }
    if (this.street === 'settled') {
      return { text: 'Hand resolving', tone: 'wait', detail: this.handLine() };
    }
    const me = this.ctx.myIndex;
    if (me < 0) {
      return { text: 'Spectating', tone: 'idle', detail: this.handLine() };
    }
    if (this.toAct === me && !this.folded[me] && !this.allIn[me]) {
      return { text: 'Your action', tone: 'turn', detail: this.handLine() };
    }
    return { text: 'Opponent’s action', tone: 'wait', detail: this.handLine() };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished' || this.outcome === null) return null;
    const me = this.ctx.myIndex;
    const myStack = this.finalStacks[me] ?? 0;
    const oppStack = this.finalStacks[1 - me] ?? 0;
    return {
      outcome: 'draw',
      title: 'Final score',
      details: [
        `Your chips: ${myStack}`,
        `Opponent chips: ${oppStack}`,
        `${this.handNo} hand${this.handNo === 1 ? '' : 's'} played`,
      ],
    };
  }

  private handLine(): string {
    return `Hand ${this.handNo}/${this.numHands || '?'} · Pot ${this.pot()}`;
  }

  private pot(): number {
    if (this.potSnapshot > 0) return this.potSnapshot;
    return this.committed[0] + this.committed[1];
  }

  protected render(): void {
    this.container.removeChildren();

    if (this.gameState.status === 'waiting') {
      const t = new Text({
        text: 'Waiting for game to start',
        style: { fontSize: 16, fill: COLORS.GRAY },
      });
      t.x = (CANVAS_W - t.width) / 2;
      t.y = CANVAS_H / 2 - t.height / 2;
      this.container.addChild(t);
      return;
    }

    if (this.gameState.status === 'finished') {
      this.renderFinal();
      return;
    }

    const me = this.ctx.myIndex >= 0 ? this.ctx.myIndex : 0;
    const opp = 1 - me;

    // Top: opponent panel.
    this.renderPlayerPanel({
      label: 'Opponent',
      stack: this.stacks[opp] ?? 0,
      bet: this.betInStreet[opp] ?? 0,
      cards: this.revealOpponent ? this.oppHole : [null, null],
      faceDown: !this.revealOpponent,
      isDealer: this.dealer === opp,
      isToAct: this.toAct === opp && !this.folded[opp] && this.street !== 'settled',
      folded: this.folded[opp],
      allIn: this.allIn[opp],
      y: 12,
    });

    // Middle: pot + community cards.
    this.renderBoard();

    // Bottom: my panel + action buttons.
    this.renderPlayerPanel({
      label: 'You',
      stack: this.stacks[me] ?? 0,
      bet: this.betInStreet[me] ?? 0,
      cards: this.hole,
      faceDown: false,
      isDealer: this.dealer === me,
      isToAct: this.toAct === me && !this.folded[me] && this.street !== 'settled',
      folded: this.folded[me],
      allIn: this.allIn[me],
      y: 240,
    });

    if (this.canAct()) {
      this.renderActions();
    }
  }

  private renderPlayerPanel(p: {
    label: string;
    stack: number;
    bet: number;
    cards: [Card | null, Card | null];
    faceDown: boolean;
    isDealer: boolean;
    isToAct: boolean;
    folded: boolean;
    allIn: boolean;
    y: number;
  }): void {
    const bg = new Graphics();
    bg.roundRect(8, p.y, CANVAS_W - 16, 88, 8);
    bg.fill(p.isToAct ? 0xfff8e1 : COLORS.LIGHT_GRAY);
    bg.stroke({ width: 1, color: COLORS.GRAY });
    this.container.addChild(bg);

    const label = new Text({
      text: p.label + (p.isDealer ? ' (D)' : ''),
      style: { fontSize: 13, fill: COLORS.GRAY, fontWeight: 'bold' },
    });
    label.x = 16;
    label.y = p.y + 6;
    this.container.addChild(label);

    let statusSuffix = '';
    if (p.folded) statusSuffix = ' · folded';
    else if (p.allIn) statusSuffix = ' · all-in';

    const stackText = new Text({
      text: `Stack ${p.stack}${statusSuffix}`,
      style: { fontSize: 14, fill: COLORS.BLACK },
    });
    stackText.x = 16;
    stackText.y = p.y + 24;
    this.container.addChild(stackText);

    if (p.bet > 0) {
      const betText = new Text({
        text: `Bet ${p.bet}`,
        style: { fontSize: 13, fill: COLORS.BLUE },
      });
      betText.x = 16;
      betText.y = p.y + 44;
      this.container.addChild(betText);
    }

    // Cards on the right side of the panel.
    const cardsX = CANVAS_W - 16 - CARD_W * 2 - 8;
    this.drawCard(cardsX, p.y + 12, p.cards[0], p.faceDown);
    this.drawCard(cardsX + CARD_W + 6, p.y + 12, p.cards[1], p.faceDown);
  }

  private renderBoard(): void {
    const y = 110;
    const potText = new Text({
      text: `Pot ${this.pot()}`,
      style: { fontSize: 16, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    potText.x = (CANVAS_W - potText.width) / 2;
    potText.y = y;
    this.container.addChild(potText);

    const slotsY = y + 26;
    const totalW = CARD_W * 5 + 6 * 4;
    const startX = (CANVAS_W - totalW) / 2;
    for (let i = 0; i < 5; i++) {
      const card = this.board[i] ?? null;
      const slotX = startX + i * (CARD_W + 6);
      this.drawCard(slotX, slotsY, card, false, !card);
    }
  }

  private drawCard(
    x: number,
    y: number,
    card: Card | null,
    faceDown: boolean,
    placeholder = false,
  ): void {
    const g = new Graphics();
    g.roundRect(x, y, CARD_W, CARD_H, 6);
    if (placeholder) {
      g.fill(0xeeeeee);
      g.stroke({ width: 1, color: 0xcccccc });
    } else if (faceDown) {
      g.fill(COLORS.BLUE);
      g.stroke({ width: 2, color: 0x0d47a1 });
    } else {
      g.fill(COLORS.WHITE);
      g.stroke({ width: 1, color: COLORS.BLACK });
    }
    this.container.addChild(g);

    if (placeholder || faceDown || !card) return;

    const color = isRedSuit(card.suit) ? COLORS.RED : COLORS.BLACK;
    const rankText = new Text({
      text: card.rank,
      style: { fontSize: 18, fill: color, fontWeight: 'bold' },
    });
    rankText.x = x + 4;
    rankText.y = y + 2;
    this.container.addChild(rankText);

    const suitText = new Text({
      text: suitGlyph(card.suit),
      style: { fontSize: 22, fill: color, fontWeight: 'bold' },
    });
    suitText.x = x + CARD_W - suitText.width - 4;
    suitText.y = y + CARD_H - suitText.height - 2;
    this.container.addChild(suitText);
  }

  private canAct(): boolean {
    if (!this.wsConnected) return false;
    if (this.gameState.status !== 'playing') return false;
    if (this.street === 'settled') return false;
    const me = this.ctx.myIndex;
    if (me < 0) return false;
    if (this.folded[me] || this.allIn[me]) return false;
    return this.toAct === me;
  }

  private renderActions(): void {
    const me = this.ctx.myIndex;
    const opp = 1 - me;
    const toCall = Math.max(0, this.betInStreet[opp] - this.betInStreet[me]);
    const stack = this.stacks[me];
    const myCommitStreet = this.betInStreet[me];
    const minRaiseInc = Math.max(this.smallBlind * 2, this.lastRaiseSize);
    const minBet = this.smallBlind * 2;
    const noBetYet = (this.betInStreet[0] ?? 0) === 0 && (this.betInStreet[1] ?? 0) === 0;

    const buttons: Array<{
      label: string;
      enabled: boolean;
      onPress: () => void;
      color: number;
    }> = [];

    buttons.push({
      label: 'Fold',
      enabled: true,
      onPress: () => this.act?.('FOLD', ''),
      color: 0xb71c1c,
    });

    if (toCall === 0) {
      buttons.push({
        label: 'Check',
        enabled: true,
        onPress: () => this.act?.('CHECK', ''),
        color: 0x2e7d32,
      });
    } else {
      buttons.push({
        label: `Call ${Math.min(toCall, stack)}`,
        enabled: stack > 0,
        onPress: () => this.act?.('CALL', ''),
        color: 0x1565c0,
      });
    }

    if (noBetYet) {
      const amt = Math.max(minBet, this.betInput);
      buttons.push({
        label: `Bet ${amt}`,
        enabled: stack >= minBet,
        onPress: () => this.act?.('BET', String(Math.min(amt, stack))),
        color: 0x6a1b9a,
      });
    } else {
      const target = Math.max(this.maxBet() + minRaiseInc, this.betInput);
      const need = target - myCommitStreet;
      buttons.push({
        label: `Raise to ${target}`,
        enabled: stack >= need && need > toCall,
        onPress: () => this.act?.('RAISE', String(target)),
        color: 0x6a1b9a,
      });
    }

    buttons.push({
      label: 'All-in',
      enabled: stack > 0,
      onPress: () => this.act?.('ALLIN', ''),
      color: 0x4a148c,
    });

    // Lay out 4 buttons across the bottom.
    const totalW = CANVAS_W - 16;
    const gap = 6;
    const btnW = (totalW - gap * (buttons.length - 1)) / buttons.length;
    const btnH = 36;
    const baseY = 340;
    buttons.forEach((b, i) => {
      const x = 8 + i * (btnW + gap);
      const g = new Graphics();
      g.roundRect(x, baseY, btnW, btnH, 8);
      g.fill(b.enabled ? b.color : 0xbdbdbd);
      this.container.addChild(g);
      if (b.enabled) {
        g.interactive = true;
        g.cursor = 'pointer';
        g.on('pointerdown', b.onPress);
      }
      const t = new Text({
        text: b.label,
        style: { fontSize: 12, fill: COLORS.WHITE, fontWeight: 'bold' },
      });
      t.x = x + (btnW - t.width) / 2;
      t.y = baseY + (btnH - t.height) / 2;
      this.container.addChild(t);
    });

    // Increment / decrement controls for the bet input.
    const stepY = baseY - 28;
    const stepLabel = new Text({
      text: `Custom amount: ${Math.max(minBet, this.betInput)}`,
      style: { fontSize: 11, fill: COLORS.GRAY },
    });
    stepLabel.x = 12;
    stepLabel.y = stepY;
    this.container.addChild(stepLabel);

    const minus = this.makeStepButton(CANVAS_W - 90, stepY - 4, '−', () => {
      this.betInput = Math.max(minBet, this.betInput - this.smallBlind * 2);
      this.refresh();
    });
    const plus = this.makeStepButton(CANVAS_W - 60, stepY - 4, '+', () => {
      this.betInput = Math.min(stack + myCommitStreet, this.betInput + this.smallBlind * 2);
      this.refresh();
    });
    const allInPreset = this.makeStepButton(CANVAS_W - 30, stepY - 4, 'M', () => {
      this.betInput = stack + myCommitStreet;
      this.refresh();
    });
    void minus;
    void plus;
    void allInPreset;
  }

  private maxBet(): number {
    return Math.max(this.betInStreet[0] ?? 0, this.betInStreet[1] ?? 0);
  }

  private makeStepButton(x: number, y: number, label: string, onPress: () => void): Graphics {
    const g = new Graphics();
    g.roundRect(x, y, 22, 22, 4);
    g.fill(COLORS.LIGHT_GRAY);
    g.stroke({ width: 1, color: COLORS.GRAY });
    g.interactive = true;
    g.cursor = 'pointer';
    g.on('pointerdown', onPress);
    this.container.addChild(g);
    const t = new Text({
      text: label,
      style: { fontSize: 14, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    t.x = x + (22 - t.width) / 2;
    t.y = y + (22 - t.height) / 2;
    this.container.addChild(t);
    return g;
  }

  private renderFinal(): void {
    const me = this.ctx.myIndex >= 0 ? this.ctx.myIndex : 0;
    const myStack = this.finalStacks[me] ?? 0;
    const oppStack = this.finalStacks[1 - me] ?? 0;

    const heading = new Text({
      text: 'Final score',
      style: { fontSize: 28, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    heading.x = (CANVAS_W - heading.width) / 2;
    heading.y = 130;
    this.container.addChild(heading);

    const stackLine = new Text({
      text: `Final chips: ${myStack} vs ${oppStack}`,
      style: { fontSize: 18, fill: COLORS.BLACK },
    });
    stackLine.x = (CANVAS_W - stackLine.width) / 2;
    stackLine.y = 180;
    this.container.addChild(stackLine);

    const handsLine = new Text({
      text: `${this.handNo} hand${this.handNo === 1 ? '' : 's'} played`,
      style: { fontSize: 14, fill: COLORS.GRAY },
    });
    handsLine.x = (CANVAS_W - handsLine.width) / 2;
    handsLine.y = 210;
    this.container.addChild(handsLine);
  }

  /**
   * Available action verbs (kept exported for tests / debugging).
   */
  public static readonly ACTIONS = ACTION_VERBS;
}
