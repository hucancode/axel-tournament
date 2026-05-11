import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

/**
 * Jar of Greed visualizer.
 *
 * Spec: judge/protocols/jar-of-greed.md.
 *
 * Layout: jar centered on the canvas, players arranged on a circle
 * around it (you anchored at the bottom, others walking clockwise from
 * there). Per round you pick how many of your coins to throw in, the
 * jar grows by the configured multiplier, and each player gets an even
 * share of the pot back.
 *
 * Hidden information: only your own contribution amount is rendered in
 * real time. Other players' amounts stay hidden until ROUND_RESULT.
 */

interface RoundRow {
  round: number;
  multiplier: number;
  jar: number;
  payout: number;
  myCoins: number;
  myContribution: number;
  /// Per-player balances post-payout. Empty when game is in blind mode.
  balances: number[];
}

const CANVAS_W = 400;
const CANVAS_H = 400;
const CENTER_X = CANVAS_W / 2;
const CENTER_Y = CANVAS_H / 2;
const PLAYER_RADIUS = 130;
const JAR_RADIUS = 56;

export class JarOfGreedGame extends BasePixiGame {
  private startingCoins = 0;
  private totalRounds = 0;
  private random = false;
  private blind = true;
  private configuredMultiplier = 0;
  private currentRound = 0;
  private playerCount = 0;
  /// Per-player balances for display. Slots without a known balance
  /// hold the starting value as a placeholder; check `coinsKnown`
  /// before showing them. The local player's slot is always known.
  private coins: number[] = [];
  /// Whether each player's slot in `coins` reflects their real current
  /// balance. Always true for the local player; true for others only
  /// after GAME_END (blind) or after the first ROUND_RESULT (open).
  private coinsKnown: boolean[] = [];
  /// Authoritative own balance, tracked from starting + payouts -
  /// contributions independently of the wire payload. Lets the local
  /// player see their own running stack even in blind mode.
  private myBalance = 0;
  /// `null` until the player has submitted their contribution this round.
  private myContribution: number | null = null;
  private contributedFlags: boolean[] = [];
  private inputAmount = 0;
  private history: RoundRow[] = [];
  private lastResult: RoundRow | null = null;
  private outcome: 'win' | 'lose' | 'draw' | null = null;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;
    const me = ctx.myIndex;

    switch (kind) {
      case 'GAME_STARTED': {
        const p = payload.split(' ');
        this.startingCoins = parseInt(p[0] ?? '', 10) || 10;
        this.totalRounds = parseInt(p[1] ?? '', 10) || 5;
        this.random = (parseInt(p[2] ?? '0', 10) || 0) !== 0;
        this.configuredMultiplier = parseFloat(p[3] ?? '') || 2;
        // blind defaults to true when omitted (matches server logic).
        this.blind = p[4] === undefined ? true : parseInt(p[4] ?? '1', 10) !== 0;
        this.playerCount = Math.max(2, ctx.players.length);
        this.coins = new Array(this.playerCount).fill(this.startingCoins);
        this.coinsKnown = new Array(this.playerCount).fill(false);
        this.myBalance = this.startingCoins;
        if (me >= 0 && me < this.playerCount) {
          this.coinsKnown[me] = true;
          this.coins[me] = this.myBalance;
        }
        this.contributedFlags = new Array(this.playerCount).fill(false);
        this.currentRound = 1;
        this.myContribution = null;
        this.inputAmount = 0;
        this.history = [];
        this.lastResult = null;
        this.outcome = null;
        this.gameState.status = 'playing';
        this.refresh();
        break;
      }
      case 'CONTRIBUTE': {
        const sp = payload.indexOf(' ');
        if (sp < 0) break;
        const pid = payload.slice(0, sp);
        const amt = parseInt(payload.slice(sp + 1).trim(), 10) || 0;
        const idx = ctx.players.indexOf(pid);
        if (idx < 0) break;
        if (idx < this.contributedFlags.length) {
          this.contributedFlags[idx] = true;
        }
        if (idx === me) {
          if (this.myContribution === null) this.myContribution = amt;
        }
        this.refresh();
        break;
      }
      case 'ROUND_RESULT': {
        const p = payload.split(' ');
        const round = parseInt(p[0] ?? '', 10) || this.currentRound;
        const mult = parseFloat(p[1] ?? '') || 0;
        const jar = parseInt(p[2] ?? '', 10) || 0;
        const payout = parseInt(p[3] ?? '', 10) || 0;
        // Update own balance: deduct what we contributed this round
        // (already locally tracked) and add the payout.
        const myAmt = this.myContribution ?? 0;
        this.myBalance = this.myBalance - myAmt + payout;
        if (me >= 0 && me < this.playerCount) {
          this.coins[me] = this.myBalance;
        }
        // Open mode: payload carries per-player balances after `payout`.
        // Blind mode: other slots stay unknown until GAME_END.
        const openBalances: number[] = [];
        if (p.length >= 4 + this.playerCount) {
          for (let i = 0; i < this.playerCount; i++) {
            openBalances.push(parseInt(p[4 + i] ?? '', 10) || 0);
          }
          this.coins = openBalances.slice();
          this.coinsKnown = new Array(this.playerCount).fill(true);
        }
        const myCoins = this.coins[me] ?? 0;
        const row: RoundRow = {
          round,
          multiplier: mult,
          jar,
          payout,
          myCoins,
          myContribution: myAmt,
          balances: openBalances,
        };
        this.history.push(row);
        this.lastResult = row;
        if (round < this.totalRounds) {
          this.currentRound = round + 1;
        }
        this.myContribution = null;
        this.contributedFlags = new Array(this.playerCount).fill(false);
        this.inputAmount = 0;
        this.refresh();
        break;
      }
      case 'GAME_END': {
        const p = payload.split(' ');
        const balances: number[] = [];
        for (let i = 0; i < this.playerCount; i++) {
          balances.push(parseInt(p[i] ?? '', 10) || 0);
        }
        this.coins = balances;
        this.coinsKnown = new Array(this.playerCount).fill(true);
        const myCoins = balances[me] ?? 0;
        const max = balances.reduce((m, c) => (c > m ? c : m), -Infinity);
        const tiedAtTop = balances.filter((c) => c === max).length;
        if (myCoins === max && tiedAtTop === 1) this.outcome = 'win';
        else if (myCoins === max) this.outcome = 'draw';
        else this.outcome = 'lose';
        this.gameState.status = 'finished';
        this.refresh();
        break;
      }
      case 'WINNER': {
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
      const detail = this.scoreLine();
      if (this.outcome === 'win') return { text: 'You Win!', tone: 'win', detail };
      if (this.outcome === 'lose') return { text: 'You Lose', tone: 'lose', detail };
      return { text: 'Draw', tone: 'draw', detail };
    }
    if (this.lastResult) {
      const r = this.lastResult;
      return {
        text: `×${this.fmtMult(r.multiplier)} · jar ${r.jar} → ${r.payout} each`,
        tone: 'turn',
        detail: this.roundLine(),
      };
    }
    if (this.myContribution !== null) {
      const remaining = this.contributedFlags.filter((c) => !c).length;
      const text =
        remaining === 0
          ? 'Resolving round'
          : `Waiting on ${remaining} player${remaining === 1 ? '' : 's'}`;
      return { text, tone: 'wait', detail: this.roundLine() };
    }
    return { text: 'Pick contribution', tone: 'turn', detail: this.roundLine() };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished' || this.outcome === null) return null;
    const me = this.ctx.myIndex;
    const myCoins = this.coins[me] ?? 0;
    const sorted = [...this.coins].sort((a, b) => b - a);
    const title = this.outcome === 'win' ? 'Victory' : this.outcome === 'lose' ? 'Defeat' : 'Draw';
    return {
      outcome: this.outcome,
      title,
      details: [
        `Your coins: ${myCoins}`,
        `Top stacks: ${sorted.slice(0, Math.min(3, sorted.length)).join(' / ')}`,
        `${this.totalRounds} round${this.totalRounds === 1 ? '' : 's'} played · ${this.playerCount} players`,
      ],
    };
  }

  private roundLine(): string {
    const modeLabel = this.random
      ? `random ≤ ×${this.fmtMult(this.configuredMultiplier)}`
      : `×${this.fmtMult(this.configuredMultiplier)}`;
    return `Round ${this.currentRound}/${this.totalRounds} · ${modeLabel} · ${this.scoreLine()}`;
  }

  private scoreLine(): string {
    const me = this.ctx.myIndex >= 0 ? this.ctx.myIndex : 0;
    const my = this.coins[me] ?? 0;
    // List others — render `?` for slots whose balance is still hidden.
    const others = this.coins
      .map((c, i) => ({ c, i, known: this.coinsKnown[i] ?? false }))
      .filter((p) => p.i !== me)
      .sort((a, b) => (a.known && b.known ? b.c - a.c : 0))
      .map((p) => (p.known ? p.c.toString() : '?'));
    if (others.length === 0) return `Coins ${my}`;
    return `Coins ${my} – others ${others.join('/')}`;
  }

  private fmtMult(m: number): string {
    if (!Number.isFinite(m)) return '?';
    const r = Math.round(m * 100) / 100;
    return r.toString();
  }

  protected render(): void {
    this.container.removeChildren();

    if (this.gameState.status === 'waiting') {
      const t = new Text({
        text: 'Waiting for game to start',
        style: { fontSize: 16, fill: COLORS.GRAY },
      });
      t.x = (CANVAS_W - t.width) / 2;
      t.y = CENTER_Y - t.height / 2;
      this.container.addChild(t);
      return;
    }

    if (this.gameState.status === 'finished') {
      this.renderFinal();
      return;
    }

    this.renderHistoryStrip();
    this.renderJar();
    this.renderPlayers();

    if (this.lastResult) {
      this.renderRoundOverlay();
    } else if (this.myContribution === null) {
      this.renderInput();
    }
  }

  private renderHistoryStrip(): void {
    if (this.history.length === 0) return;
    const cellW = Math.min(38, Math.floor((CANVAS_W - 16) / this.history.length));
    const totalW = this.history.length * cellW;
    const startX = (CANVAS_W - totalW) / 2;
    for (let i = 0; i < this.history.length; i++) {
      const r = this.history[i];
      const cx = startX + i * cellW + cellW / 2;
      const positive = r.payout >= r.myContribution;
      const bg = new Graphics();
      bg.roundRect(cx - cellW / 2 + 1, 4, cellW - 2, 26, 4);
      bg.fill(COLORS.LIGHT_GRAY);
      bg.stroke({ width: 1, color: positive ? COLORS.GREEN : COLORS.RED });
      this.container.addChild(bg);

      const mult = new Text({
        text: `×${this.fmtMult(r.multiplier)}`,
        style: { fontSize: 10, fill: COLORS.BLACK, fontWeight: 'bold' },
      });
      mult.x = cx - mult.width / 2;
      mult.y = 6;
      this.container.addChild(mult);

      const delta = r.payout - r.myContribution;
      const sign = delta >= 0 ? '+' : '';
      const deltaText = new Text({
        text: `${sign}${delta}`,
        style: { fontSize: 9, fill: positive ? COLORS.GREEN : COLORS.RED },
      });
      deltaText.x = cx - deltaText.width / 2;
      deltaText.y = 18;
      this.container.addChild(deltaText);
    }
  }

  private renderJar(): void {
    const jar = new Graphics();
    jar.roundRect(CENTER_X - JAR_RADIUS, CENTER_Y - JAR_RADIUS + 14, JAR_RADIUS * 2, JAR_RADIUS * 2, 16);
    jar.fill(0xfff3c4);
    jar.stroke({ width: 3, color: 0xb8860b });
    this.container.addChild(jar);

    const rim = new Graphics();
    rim.roundRect(CENTER_X - JAR_RADIUS - 6, CENTER_Y - JAR_RADIUS + 6, JAR_RADIUS * 2 + 12, 14, 6);
    rim.fill(0xe6c065);
    rim.stroke({ width: 2, color: 0x8b6914 });
    this.container.addChild(rim);

    const myAmt = this.myContribution ?? 0;
    const knownTotal = myAmt;
    const fillRatio = Math.min(
      1,
      knownTotal / Math.max(1, this.coins.reduce((s, c) => s + c, 0) + myAmt + 1),
    );
    if (myAmt > 0) {
      const fillH = Math.max(6, fillRatio * (JAR_RADIUS * 2 - 14));
      const fill = new Graphics();
      fill.roundRect(
        CENTER_X - JAR_RADIUS + 6,
        CENTER_Y + JAR_RADIUS - 8 - fillH,
        JAR_RADIUS * 2 - 12,
        fillH,
        12,
      );
      fill.fill(0xffd54f);
      this.container.addChild(fill);

      const myLabel = new Text({
        text: `you +${myAmt}`,
        style: { fontSize: 12, fill: COLORS.BLACK, fontWeight: 'bold' },
      });
      myLabel.x = CENTER_X - myLabel.width / 2;
      myLabel.y = CENTER_Y - 6;
      this.container.addChild(myLabel);
    }

    // Count opponents who have already submitted (amount unknown).
    const me = this.ctx.myIndex;
    const oppCommitted = this.contributedFlags.reduce(
      (n, c, i) => (c && i !== me ? n + 1 : n),
      0,
    );
    if (oppCommitted > 0) {
      const oppLabel = new Text({
        text: `+${oppCommitted}× ?`,
        style: { fontSize: 11, fill: COLORS.GRAY },
      });
      oppLabel.x = CENTER_X - oppLabel.width / 2;
      oppLabel.y = CENTER_Y + 12;
      this.container.addChild(oppLabel);
    }

    const modeLabel = this.random
      ? `×1 – ×${this.fmtMult(this.configuredMultiplier)} (random)`
      : `×${this.fmtMult(this.configuredMultiplier)}`;
    const tag = new Text({
      text: modeLabel,
      style: { fontSize: 11, fill: COLORS.GRAY },
    });
    tag.x = CENTER_X - tag.width / 2;
    tag.y = CENTER_Y + JAR_RADIUS + 18;
    this.container.addChild(tag);
  }

  private renderPlayers(): void {
    const me = this.ctx.myIndex >= 0 ? this.ctx.myIndex : 0;
    const players = this.ctx.players;
    const n = Math.max(1, players.length || this.playerCount || 2);
    // Shrink node + radius slightly when the table is crowded.
    const radius = n <= 2 ? PLAYER_RADIUS : PLAYER_RADIUS - Math.min(20, (n - 2) * 4);
    const nodeW = n <= 4 ? 110 : 88;
    const nodeH = n <= 4 ? 56 : 48;
    for (let i = 0; i < n; i++) {
      // Self always at the bottom (angle = π/2). Others walk clockwise.
      const orderRel = (i - me + n) % n;
      const angle = Math.PI / 2 + (orderRel * 2 * Math.PI) / n;
      const px = CENTER_X + Math.cos(angle) * radius;
      const py = CENTER_Y + Math.sin(angle) * radius;
      const isMe = i === me;
      const committed = this.contributedFlags[i] ?? false;
      const label = isMe ? 'You' : players[i] ? this.shortName(players[i]) : `P${i + 1}`;
      const known = this.coinsKnown[i] ?? false;
      this.drawPlayerNode({
        cx: px,
        cy: py,
        w: nodeW,
        h: nodeH,
        label,
        coins: this.coins[i] ?? 0,
        coinsKnown: known,
        committed,
        amount: isMe ? this.myContribution ?? null : null,
      });
    }
  }

  private shortName(pid: string): string {
    if (pid.length <= 10) return pid;
    return `${pid.slice(0, 7)}…`;
  }

  private drawPlayerNode(p: {
    cx: number;
    cy: number;
    w: number;
    h: number;
    label: string;
    coins: number;
    coinsKnown: boolean;
    committed: boolean;
    amount: number | null;
  }): void {
    const x = p.cx - p.w / 2;
    const y = p.cy - p.h / 2;
    const bg = new Graphics();
    bg.roundRect(x, y, p.w, p.h, 10);
    bg.fill(p.committed ? 0xc8e6c9 : 0xe3f2fd);
    bg.stroke({ width: 2, color: p.committed ? COLORS.GREEN : COLORS.BLUE });
    this.container.addChild(bg);

    const label = new Text({
      text: p.label,
      style: { fontSize: 10, fill: COLORS.GRAY, fontWeight: 'bold' },
    });
    label.x = x + (p.w - label.width) / 2;
    label.y = y + 3;
    this.container.addChild(label);

    const coinsText = new Text({
      text: p.coinsKnown ? `${p.coins} 🪙` : `? 🪙`,
      style: {
        fontSize: 14,
        fill: p.coinsKnown ? COLORS.BLACK : COLORS.GRAY,
        fontWeight: 'bold',
      },
    });
    coinsText.x = x + (p.w - coinsText.width) / 2;
    coinsText.y = y + 16;
    this.container.addChild(coinsText);

    let stateText: string;
    if (p.committed && p.amount !== null) stateText = `put +${p.amount}`;
    else if (p.committed) stateText = 'submitted';
    else stateText = 'choosing…';
    const st = new Text({
      text: stateText,
      style: { fontSize: 9, fill: COLORS.GRAY },
    });
    st.x = x + (p.w - st.width) / 2;
    st.y = y + p.h - 14;
    this.container.addChild(st);
  }

  private renderInput(): void {
    const me = this.ctx.myIndex;
    if (me < 0) return;
    const max = this.coins[me] ?? 0;
    if (max <= 0) {
      const submit = this.makeButton(140, 350, 120, 36, 'Submit (0)', COLORS.GREEN, () => {
        this.submitContribution(0);
      });
      void submit;
      return;
    }

    const value = Math.max(0, Math.min(max, this.inputAmount));

    const sliderY = 326;
    const sliderX = 30;
    const sliderW = CANVAS_W - 60;
    const track = new Graphics();
    track.roundRect(sliderX, sliderY, sliderW, 8, 4);
    track.fill(COLORS.LIGHT_GRAY);
    track.stroke({ width: 1, color: COLORS.GRAY });
    track.interactive = true;
    track.cursor = 'pointer';
    track.on('pointerdown', (e) => {
      const local = e.global.x - sliderX;
      const ratio = Math.max(0, Math.min(1, local / sliderW));
      this.inputAmount = Math.round(ratio * max);
      this.refresh();
    });
    this.container.addChild(track);

    const knobX = sliderX + (value / max) * sliderW;
    const knob = new Graphics();
    knob.circle(knobX, sliderY + 4, 9);
    knob.fill(COLORS.BLUE);
    knob.stroke({ width: 2, color: 0x0d47a1 });
    this.container.addChild(knob);

    const valLabel = new Text({
      text: `Contribute: ${value} / ${max}`,
      style: { fontSize: 14, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    valLabel.x = (CANVAS_W - valLabel.width) / 2;
    valLabel.y = sliderY - 22;
    this.container.addChild(valLabel);

    this.makeStepButton(sliderX - 24, sliderY - 6, '−', () => {
      this.inputAmount = Math.max(0, value - 1);
      this.refresh();
    });
    this.makeStepButton(sliderX + sliderW + 4, sliderY - 6, '+', () => {
      this.inputAmount = Math.min(max, value + 1);
      this.refresh();
    });

    const allBtn = this.makeButton(20, 350, 90, 32, 'All in', 0xb71c1c, () => {
      this.inputAmount = max;
      this.refresh();
    });
    const noneBtn = this.makeButton(120, 350, 80, 32, 'Nothing', COLORS.GRAY, () => {
      this.inputAmount = 0;
      this.refresh();
    });
    const submit = this.makeButton(
      210,
      350,
      170,
      32,
      `Submit (${value})`,
      COLORS.GREEN,
      () => this.submitContribution(value),
    );
    void allBtn;
    void noneBtn;
    void submit;
  }

  private renderRoundOverlay(): void {
    const r = this.lastResult!;
    const card = new Graphics();
    card.roundRect(40, 250, CANVAS_W - 80, 110, 12);
    card.fill(0xfffde7);
    card.stroke({ width: 2, color: 0xfbc02d });
    this.container.addChild(card);

    const title = new Text({
      text: `Round ${r.round} resolved`,
      style: { fontSize: 14, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    title.x = (CANVAS_W - title.width) / 2;
    title.y = 258;
    this.container.addChild(title);

    const line1 = new Text({
      text: `Jar ${r.jar} × ${this.fmtMult(r.multiplier)} → payout ${r.payout} each`,
      style: { fontSize: 13, fill: COLORS.BLACK },
    });
    line1.x = (CANVAS_W - line1.width) / 2;
    line1.y = 280;
    this.container.addChild(line1);

    const delta = r.payout - r.myContribution;
    const sign = delta >= 0 ? '+' : '';
    const line2 = new Text({
      text: `You: contributed ${r.myContribution}, net ${sign}${delta}`,
      style: { fontSize: 13, fill: delta >= 0 ? COLORS.GREEN : COLORS.RED, fontWeight: 'bold' },
    });
    line2.x = (CANVAS_W - line2.width) / 2;
    line2.y = 302;
    this.container.addChild(line2);

    const next = this.makeButton(
      (CANVAS_W - 140) / 2,
      328,
      140,
      28,
      r.round >= this.totalRounds ? 'Final result →' : 'Next round →',
      COLORS.BLUE,
      () => {
        this.lastResult = null;
        this.refresh();
      },
    );
    void next;
  }

  private renderFinal(): void {
    const me = this.ctx.myIndex >= 0 ? this.ctx.myIndex : 0;
    const my = this.coins[me] ?? 0;

    this.renderHistoryStrip();
    this.renderJar();
    this.renderPlayers();

    const card = new Graphics();
    card.roundRect(40, 250, CANVAS_W - 80, 120, 12);
    card.fill(COLORS.WHITE);
    card.stroke({ width: 2, color: COLORS.BLACK });
    this.container.addChild(card);

    const heading = new Text({
      text: this.outcome === 'win' ? 'You win!' : this.outcome === 'lose' ? 'You lose' : 'Draw',
      style: { fontSize: 24, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    heading.x = (CANVAS_W - heading.width) / 2;
    heading.y = 258;
    this.container.addChild(heading);

    const score = new Text({
      text: `Final coins — you: ${my}`,
      style: { fontSize: 16, fill: COLORS.BLACK },
    });
    score.x = (CANVAS_W - score.width) / 2;
    score.y = 296;
    this.container.addChild(score);

    const others = this.coins
      .map((c, i) => ({ c, i }))
      .filter((p) => p.i !== me)
      .sort((a, b) => b.c - a.c)
      .map((p) => p.c)
      .join(' / ');
    const oppLine = new Text({
      text: others ? `Others: ${others}` : '',
      style: { fontSize: 13, fill: COLORS.GRAY },
    });
    oppLine.x = (CANVAS_W - oppLine.width) / 2;
    oppLine.y = 320;
    this.container.addChild(oppLine);

    const rounds = new Text({
      text: `${this.totalRounds} rounds · ${this.playerCount} players`,
      style: { fontSize: 12, fill: COLORS.GRAY },
    });
    rounds.x = (CANVAS_W - rounds.width) / 2;
    rounds.y = 342;
    this.container.addChild(rounds);
  }

  private makeButton(
    x: number,
    y: number,
    w: number,
    h: number,
    label: string,
    color: number,
    onPress: () => void,
  ): Graphics {
    const g = new Graphics();
    g.roundRect(x, y, w, h, 8);
    g.fill(color);
    g.interactive = true;
    g.cursor = 'pointer';
    g.on('pointerdown', onPress);
    this.container.addChild(g);
    const t = new Text({
      text: label,
      style: { fontSize: 13, fill: COLORS.WHITE, fontWeight: 'bold' },
    });
    t.x = x + (w - t.width) / 2;
    t.y = y + (h - t.height) / 2;
    this.container.addChild(t);
    return g;
  }

  private makeStepButton(x: number, y: number, label: string, onPress: () => void): void {
    const g = new Graphics();
    g.roundRect(x, y, 20, 20, 4);
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
    t.x = x + (20 - t.width) / 2;
    t.y = y + (20 - t.height) / 2;
    this.container.addChild(t);
  }

  private submitContribution(amount: number): void {
    if (!this.wsConnected) return;
    if (this.gameState.status !== 'playing') return;
    if (this.myContribution !== null) return;
    this.myContribution = amount;
    this.act?.('CONTRIBUTE', String(amount));
    this.refresh();
  }
}
