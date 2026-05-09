import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext } from './BasePixiGame';
import { COLORS } from './types';

type Choice = 'C' | 'D';

/** Spec: judge/protocols/prisoners-dilemma.md. */
export class PrisonersDilemmaGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private opponentChoice: Choice | null = null;
  private scores = { player: 0, opponent: 0 };

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;
    const me = ctx.myIndex;

    switch (kind) {
      case 'GAME_STARTED':
        this.gameState.status = 'playing';
        this.scores = { player: 0, opponent: 0 };
        this.myChoice = null;
        this.opponentChoice = null;
        this.render();
        break;
      case 'ROUND_RESULT': {
        const p = payload.split(' ');
        const m0 = p[1] as Choice;
        const m1 = p[2] as Choice;
        const s0 = parseInt(p[3], 10);
        const s1 = parseInt(p[4], 10);
        this.myChoice = me === 0 ? m0 : m1;
        this.opponentChoice = me === 0 ? m1 : m0;
        this.scores = { player: me === 0 ? s0 : s1, opponent: me === 0 ? s1 : s0 };
        this.render();
        setTimeout(() => {
          this.myChoice = null;
          this.opponentChoice = null;
          this.render();
        }, 2000);
        break;
      }
      case 'GAME_END':
        this.gameState.status = 'finished';
        this.gameState.result = `Final Score: ${this.scores.player}`;
        this.render();
        break;
    }
  }

  protected render(): void {
    this.container.removeChildren();

    const status = new Text({
      text: this.getStatusText(),
      style: { fontSize: 16, fill: COLORS.BLACK },
    });
    status.x = 200 - status.width / 2;
    status.y = 20;
    this.container.addChild(status);

    if (this.myChoice && this.opponentChoice) {
      this.renderResult();
    } else if (this.gameState.status === 'playing' && !this.myChoice) {
      this.renderChoices();
    }
  }

  private renderChoices(): void {
    const coopButton = new Graphics();
    coopButton.roundRect(80, 150, 100, 60, 8);
    coopButton.fill(COLORS.GREEN);
    coopButton.interactive = true;
    coopButton.cursor = 'pointer';
    coopButton.on('pointerdown', () => this.makeChoice('C'));
    this.container.addChild(coopButton);

    const coopText = new Text({
      text: '🤝\nCooperate',
      style: { fontSize: 14, fill: COLORS.WHITE, align: 'center' },
    });
    coopText.x = 130 - coopText.width / 2;
    coopText.y = 165;
    this.container.addChild(coopText);

    const defectButton = new Graphics();
    defectButton.roundRect(220, 150, 100, 60, 8);
    defectButton.fill(COLORS.RED);
    defectButton.interactive = true;
    defectButton.cursor = 'pointer';
    defectButton.on('pointerdown', () => this.makeChoice('D'));
    this.container.addChild(defectButton);

    const defectText = new Text({
      text: '⚔️\nDefect',
      style: { fontSize: 14, fill: COLORS.WHITE, align: 'center' },
    });
    defectText.x = 270 - defectText.width / 2;
    defectText.y = 165;
    this.container.addChild(defectText);
  }

  private renderResult(): void {
    const myEmoji = this.myChoice === 'C' ? '🤝' : '⚔️';
    const oppEmoji = this.opponentChoice === 'C' ? '🤝' : '⚔️';

    const myText = new Text({ text: myEmoji, style: { fontSize: 48 } });
    myText.x = 120;
    myText.y = 150;
    this.container.addChild(myText);

    const vsText = new Text({ text: 'VS', style: { fontSize: 24, fill: COLORS.GRAY } });
    vsText.x = 200 - vsText.width / 2;
    vsText.y = 170;
    this.container.addChild(vsText);

    const oppText = new Text({ text: oppEmoji, style: { fontSize: 48 } });
    oppText.x = 280;
    oppText.y = 150;
    this.container.addChild(oppText);

    const scoreText = new Text({
      text: `Round complete! Scores: You ${this.scores.player}, Opponent ${this.scores.opponent}`,
      style: { fontSize: 14, fill: COLORS.BLACK },
    });
    scoreText.x = 200 - scoreText.width / 2;
    scoreText.y = 250;
    this.container.addChild(scoreText);
  }

  private makeChoice(choice: Choice): void {
    if (!this.wsConnected || this.gameState.status !== 'playing' || this.myChoice) return;
    this.myChoice = choice;
    this.sendMove(choice);
    this.render();
  }

  private getStatusText(): string {
    if (this.gameState.status === 'waiting') return 'Waiting for players...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    if (this.myChoice) return 'Waiting for round result...';
    return `Score: ${this.scores.player} - ${this.scores.opponent}`;
  }
}
