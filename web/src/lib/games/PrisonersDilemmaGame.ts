import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

type Choice = 'C' | 'D';

/** Spec: judge/protocols/prisoners-dilemma.md. */
export class PrisonersDilemmaGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private opponentChoice: Choice | null = null;
  private scores = { player: 0, opponent: 0 };
  private currentRound = 0;
  private totalRounds = 0;
  private revealing = false;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;
    const me = ctx.myIndex;

    switch (kind) {
      case 'GAME_STARTED':
        this.gameState.status = 'playing';
        this.scores = { player: 0, opponent: 0 };
        this.currentRound = 1;
        this.totalRounds = parseInt(payload, 10) || 10;
        this.myChoice = null;
        this.opponentChoice = null;
        this.revealing = false;
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
        this.scores = { player: me === 0 ? s0 : s1, opponent: me === 0 ? s1 : s0 };
        this.currentRound = round;
        this.revealing = true;
        this.refresh();
        setTimeout(() => {
          this.myChoice = null;
          this.opponentChoice = null;
          this.revealing = false;
          this.currentRound = round + 1;
          this.refresh();
        }, 2000);
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
    if (this.revealing && this.myChoice && this.opponentChoice) {
      const m = this.myChoice;
      const o = this.opponentChoice;
      let text = '';
      let tone: GameStatus['tone'] = 'turn';
      if (m === 'C' && o === 'C') { text = 'Mutual cooperation'; tone = 'win'; }
      else if (m === 'D' && o === 'D') { text = 'Mutual defection'; tone = 'draw'; }
      else if (m === 'D' && o === 'C') { text = 'You defected'; tone = 'win'; }
      else { text = 'Opponent defected'; tone = 'lose'; }
      return { text, tone, detail: this.roundLine() };
    }
    if (this.myChoice) {
      return { text: 'Waiting for opponent', tone: 'wait', detail: this.roundLine() };
    }
    return { text: 'Cooperate or defect?', tone: 'turn', detail: this.roundLine() };
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
    if (this.revealing && this.myChoice && this.opponentChoice) {
      this.renderResult();
    } else if (this.myChoice) {
      this.renderWaiting();
    } else if (this.gameState.status === 'playing') {
      this.renderChoices();
    }
  }

  private renderChoices(): void {
    const coopButton = new Graphics();
    coopButton.roundRect(60, 150, 130, 100, 12);
    coopButton.fill(COLORS.GREEN);
    coopButton.stroke({ width: 3, color: 0x2e7d32 });
    coopButton.interactive = true;
    coopButton.cursor = 'pointer';
    coopButton.on('pointerdown', () => this.makeChoice('C'));
    coopButton.on('pointerover', () => { coopButton.tint = 0xdddddd; });
    coopButton.on('pointerout', () => { coopButton.tint = 0xffffff; });
    this.container.addChild(coopButton);

    const coopEmoji = new Text({ text: '🤝', style: { fontSize: 40 } });
    coopEmoji.x = 125 - coopEmoji.width / 2;
    coopEmoji.y = 165;
    this.container.addChild(coopEmoji);

    const coopText = new Text({
      text: 'Cooperate',
      style: { fontSize: 16, fill: COLORS.WHITE, fontWeight: 'bold' },
    });
    coopText.x = 125 - coopText.width / 2;
    coopText.y = 215;
    this.container.addChild(coopText);

    const defectButton = new Graphics();
    defectButton.roundRect(210, 150, 130, 100, 12);
    defectButton.fill(COLORS.RED);
    defectButton.stroke({ width: 3, color: 0xb71c1c });
    defectButton.interactive = true;
    defectButton.cursor = 'pointer';
    defectButton.on('pointerdown', () => this.makeChoice('D'));
    defectButton.on('pointerover', () => { defectButton.tint = 0xdddddd; });
    defectButton.on('pointerout', () => { defectButton.tint = 0xffffff; });
    this.container.addChild(defectButton);

    const defectEmoji = new Text({ text: '⚔️', style: { fontSize: 40 } });
    defectEmoji.x = 275 - defectEmoji.width / 2;
    defectEmoji.y = 165;
    this.container.addChild(defectEmoji);

    const defectText = new Text({
      text: 'Defect',
      style: { fontSize: 16, fill: COLORS.WHITE, fontWeight: 'bold' },
    });
    defectText.x = 275 - defectText.width / 2;
    defectText.y = 215;
    this.container.addChild(defectText);
  }

  private renderResult(): void {
    const myEmoji = this.myChoice === 'C' ? '🤝' : '⚔️';
    const oppEmoji = this.opponentChoice === 'C' ? '🤝' : '⚔️';

    const youLabel = new Text({ text: 'You', style: { fontSize: 14, fill: COLORS.GRAY } });
    youLabel.x = 100 - youLabel.width / 2;
    youLabel.y = 110;
    this.container.addChild(youLabel);

    const myText = new Text({ text: myEmoji, style: { fontSize: 72 } });
    myText.x = 100 - myText.width / 2;
    myText.y = 140;
    this.container.addChild(myText);

    const vs = new Text({ text: 'VS', style: { fontSize: 28, fill: COLORS.GRAY } });
    vs.x = 200 - vs.width / 2;
    vs.y = 175;
    this.container.addChild(vs);

    const oppLabel = new Text({ text: 'Opponent', style: { fontSize: 14, fill: COLORS.GRAY } });
    oppLabel.x = 300 - oppLabel.width / 2;
    oppLabel.y = 110;
    this.container.addChild(oppLabel);

    const oppText = new Text({ text: oppEmoji, style: { fontSize: 72 } });
    oppText.x = 300 - oppText.width / 2;
    oppText.y = 140;
    this.container.addChild(oppText);
  }

  private renderWaiting(): void {
    const myEmoji = this.myChoice === 'C' ? '🤝' : '⚔️';

    const youLabel = new Text({
      text: this.myChoice === 'C' ? 'You cooperated' : 'You defected',
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
    this.myChoice = choice;
    this.sendMove(choice);
    this.refresh();
  }
}
