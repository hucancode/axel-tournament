import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

type Choice = 'ROCK' | 'PAPER' | 'SCISSORS';

/** Spec: judge/protocols/rock-paper-scissors.md. */
export class RockPaperScissorsGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private opponentChoice: Choice | null = null;
  private roundResult: 'WIN' | 'LOSE' | 'DRAW' | null = null;
  private scores = { player: 0, opponent: 0 };
  private currentRound = 0;
  private totalRounds = 0;

  private choices = [
    { value: 'ROCK' as Choice, emoji: '🪨', label: 'Rock', x: 80 },
    { value: 'PAPER' as Choice, emoji: '📄', label: 'Paper', x: 200 },
    { value: 'SCISSORS' as Choice, emoji: '✂️', label: 'Scissors', x: 320 },
  ];

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;
    const me = ctx.myIndex;

    switch (kind) {
      case 'GAME_STARTED':
        this.gameState.status = 'playing';
        this.scores = { player: 0, opponent: 0 };
        this.currentRound = 1;
        this.totalRounds = parseInt(payload, 10) || 5;
        this.myChoice = null;
        this.opponentChoice = null;
        this.roundResult = null;
        this.refresh();
        break;
      case 'ROUND_RESULT': {
        const p = payload.split(' ');
        const round = parseInt(p[0], 10);
        const m0 = p[1] as Choice;
        const m1 = p[2] as Choice;
        const s0 = parseInt(p[3], 10);
        const s1 = parseInt(p[4], 10);
        this.myChoice = me === 0 ? m0 : m1;
        this.opponentChoice = me === 0 ? m1 : m0;
        const newScores = { player: me === 0 ? s0 : s1, opponent: me === 0 ? s1 : s0 };
        if (newScores.player > this.scores.player) this.roundResult = 'WIN';
        else if (newScores.opponent > this.scores.opponent) this.roundResult = 'LOSE';
        else this.roundResult = 'DRAW';
        this.scores = newScores;
        this.currentRound = round;
        this.refresh();
        setTimeout(() => {
          this.myChoice = null;
          this.opponentChoice = null;
          this.roundResult = null;
          this.currentRound = round + 1;
          this.refresh();
        }, 1500);
        break;
      }
      case 'GAME_END': {
        const p = payload.split(' ');
        const s0 = parseInt(p[0], 10);
        const s1 = parseInt(p[1], 10);
        this.scores = { player: me === 0 ? s0 : s1, opponent: me === 0 ? s1 : s0 };
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
      const o = this.finalOutcome();
      if (o === 'win') return { text: 'You Win!', tone: 'win', detail: this.scoreLine() };
      if (o === 'lose') return { text: 'You Lose', tone: 'lose', detail: this.scoreLine() };
      return { text: 'Draw', tone: 'draw', detail: this.scoreLine() };
    }
    if (this.roundResult) {
      const text =
        this.roundResult === 'WIN' ? 'Round won' :
        this.roundResult === 'LOSE' ? 'Round lost' : 'Round draw';
      const tone =
        this.roundResult === 'WIN' ? 'win' :
        this.roundResult === 'LOSE' ? 'lose' : 'draw';
      return { text, tone, detail: this.roundLine() };
    }
    if (this.myChoice) {
      return { text: 'Waiting for opponent', tone: 'wait', detail: this.roundLine() };
    }
    return { text: 'Make your move', tone: 'turn', detail: this.roundLine() };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished') return null;
    const o = this.finalOutcome();
    const title = o === 'win' ? 'Victory' : o === 'lose' ? 'Defeat' : 'Draw';
    return {
      outcome: o,
      title,
      details: [
        `Final score ${this.scores.player} – ${this.scores.opponent}`,
        `${this.totalRounds} rounds played`,
      ],
    };
  }

  private finalOutcome(): 'win' | 'lose' | 'draw' {
    if (this.scores.player > this.scores.opponent) return 'win';
    if (this.scores.player < this.scores.opponent) return 'lose';
    return 'draw';
  }

  private scoreLine(): string {
    return `Score ${this.scores.player} – ${this.scores.opponent}`;
  }

  private roundLine(): string {
    if (this.totalRounds) {
      return `Round ${this.currentRound} of ${this.totalRounds} · ${this.scoreLine()}`;
    }
    return `Round ${this.currentRound} · ${this.scoreLine()}`;
  }

  protected render(): void {
    this.container.removeChildren();

    if (this.gameState.status === 'finished') {
      this.renderFinalReveal();
      return;
    }
    if (this.myChoice && this.opponentChoice && this.roundResult) {
      this.renderResult();
    } else if (this.myChoice) {
      this.renderWaiting();
    } else if (this.gameState.status === 'playing') {
      this.renderChoices();
    }
  }

  private renderChoices(): void {
    this.choices.forEach((choice) => {
      const button = new Graphics();
      button.circle(choice.x, 200, 50);
      button.fill(COLORS.LIGHT_GRAY);
      button.stroke({ width: 3, color: COLORS.GRAY });
      button.interactive = true;
      button.cursor = 'pointer';
      button.on('pointerdown', () => this.makeChoice(choice.value));
      button.on('pointerover', () => {
        button.tint = 0xdddddd;
      });
      button.on('pointerout', () => {
        button.tint = 0xffffff;
      });
      this.container.addChild(button);

      const emoji = new Text({ text: choice.emoji, style: { fontSize: 40 } });
      emoji.x = choice.x - emoji.width / 2;
      emoji.y = 200 - emoji.height / 2;
      this.container.addChild(emoji);

      const label = new Text({
        text: choice.label,
        style: { fontSize: 14, fill: COLORS.BLACK },
      });
      label.x = choice.x - label.width / 2;
      label.y = 270;
      this.container.addChild(label);
    });
  }

  private renderResult(): void {
    const myEmoji = this.choices.find((c) => c.value === this.myChoice)?.emoji ?? '';
    const oppEmoji = this.choices.find((c) => c.value === this.opponentChoice)?.emoji ?? '';

    const youLabel = new Text({
      text: 'You',
      style: { fontSize: 14, fill: COLORS.GRAY },
    });
    youLabel.x = 100 - youLabel.width / 2;
    youLabel.y = 100;
    this.container.addChild(youLabel);

    const myText = new Text({ text: myEmoji, style: { fontSize: 72 } });
    myText.x = 100 - myText.width / 2;
    myText.y = 130;
    this.container.addChild(myText);

    const vs = new Text({ text: 'VS', style: { fontSize: 28, fill: COLORS.GRAY } });
    vs.x = 200 - vs.width / 2;
    vs.y = 170;
    this.container.addChild(vs);

    const oppLabel = new Text({
      text: 'Opponent',
      style: { fontSize: 14, fill: COLORS.GRAY },
    });
    oppLabel.x = 300 - oppLabel.width / 2;
    oppLabel.y = 100;
    this.container.addChild(oppLabel);

    const oppText = new Text({ text: oppEmoji, style: { fontSize: 72 } });
    oppText.x = 300 - oppText.width / 2;
    oppText.y = 130;
    this.container.addChild(oppText);
  }

  private renderWaiting(): void {
    const myEmoji = this.choices.find((c) => c.value === this.myChoice)?.emoji ?? '';

    const youLabel = new Text({
      text: 'Your move',
      style: { fontSize: 16, fill: COLORS.GRAY },
    });
    youLabel.x = 200 - youLabel.width / 2;
    youLabel.y = 130;
    this.container.addChild(youLabel);

    const myText = new Text({ text: myEmoji, style: { fontSize: 80 } });
    myText.x = 200 - myText.width / 2;
    myText.y = 160;
    this.container.addChild(myText);
  }

  private renderFinalReveal(): void {
    const final = new Text({
      text: `${this.scores.player} – ${this.scores.opponent}`,
      style: { fontSize: 64, fill: COLORS.BLACK, fontWeight: 'bold' },
    });
    final.x = 200 - final.width / 2;
    final.y = 170;
    this.container.addChild(final);
  }

  private makeChoice(choice: Choice): void {
    if (!this.wsConnected || this.gameState.status !== 'playing' || this.myChoice) return;
    if (this.roundResult !== null) return;
    this.myChoice = choice;
    this.sendMove(choice);
    this.refresh();
  }
}
