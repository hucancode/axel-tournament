import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext } from './BasePixiGame';
import { COLORS } from './types';

type Choice = 'ROCK' | 'PAPER' | 'SCISSORS';

/** Spec: judge/protocols/rock-paper-scissors.md. */
export class RockPaperScissorsGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private roundResult: 'WIN' | 'LOSE' | 'DRAW' | null = null;
  private scores = { player: 0, opponent: 0 };
  private currentRound = 0;

  private choices = [
    { value: 'ROCK' as Choice, emoji: '🪨', x: 80 },
    { value: 'PAPER' as Choice, emoji: '📄', x: 200 },
    { value: 'SCISSORS' as Choice, emoji: '✂️', x: 320 },
  ];

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;
    const me = ctx.myIndex;

    switch (kind) {
      case 'GAME_STARTED':
        this.gameState.status = 'playing';
        this.scores = { player: 0, opponent: 0 };
        this.currentRound = 1;
        this.myChoice = null;
        this.roundResult = null;
        this.render();
        break;
      case 'ROUND_RESULT': {
        const p = payload.split(' ');
        const round = parseInt(p[0], 10);
        const s0 = parseInt(p[3], 10);
        const s1 = parseInt(p[4], 10);
        const newScores = { player: me === 0 ? s0 : s1, opponent: me === 0 ? s1 : s0 };
        if (newScores.player > this.scores.player) this.roundResult = 'WIN';
        else if (newScores.opponent > this.scores.opponent) this.roundResult = 'LOSE';
        else this.roundResult = 'DRAW';
        this.scores = newScores;
        this.currentRound = round;
        this.render();
        setTimeout(() => {
          this.myChoice = null;
          this.roundResult = null;
          this.currentRound = round + 1;
          this.render();
        }, 500);
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

    const status = new Text({ text: this.getStatusText(), style: { fontSize: 16, fill: COLORS.BLACK } });
    status.x = 200 - status.width / 2;
    status.y = 20;
    this.container.addChild(status);

    if (this.gameState.status === 'playing' && !this.myChoice) {
      this.renderChoices();
    } else if (this.myChoice && this.roundResult) {
      this.renderResult();
    } else if (this.myChoice) {
      this.renderWaiting();
    }
  }

  private renderChoices(): void {
    this.choices.forEach((choice) => {
      const button = new Graphics();
      button.circle(choice.x, 200, 40);
      button.fill(COLORS.LIGHT_GRAY);
      button.stroke({ width: 2, color: COLORS.GRAY });
      button.interactive = true;
      button.cursor = 'pointer';
      button.on('pointerdown', () => this.makeChoice(choice.value));
      this.container.addChild(button);

      const text = new Text({ text: choice.emoji, style: { fontSize: 32 } });
      text.x = choice.x - text.width / 2;
      text.y = 200 - text.height / 2;
      this.container.addChild(text);
    });
  }

  private renderResult(): void {
    const myEmoji = this.choices.find((c) => c.value === this.myChoice)?.emoji || '';
    const myText = new Text({
      text: `Your choice: ${myEmoji}`,
      style: { fontSize: 32, fill: COLORS.BLACK },
    });
    myText.x = 200 - myText.width / 2;
    myText.y = 120;
    this.container.addChild(myText);

    if (this.roundResult) {
      const label =
        this.roundResult === 'WIN' ? 'You Win This Round!' :
        this.roundResult === 'LOSE' ? 'You Lose This Round!' : 'Draw!';
      const color =
        this.roundResult === 'WIN' ? COLORS.GREEN :
        this.roundResult === 'LOSE' ? COLORS.RED : COLORS.GRAY;
      const result = new Text({ text: label, style: { fontSize: 24, fill: color } });
      result.x = 200 - result.width / 2;
      result.y = 180;
      this.container.addChild(result);
    }

    const scoreText = new Text({
      text: `Score: ${this.scores.player} - ${this.scores.opponent}`,
      style: { fontSize: 18, fill: COLORS.BLACK },
    });
    scoreText.x = 200 - scoreText.width / 2;
    scoreText.y = 230;
    this.container.addChild(scoreText);
  }

  private renderWaiting(): void {
    const myEmoji = this.choices.find((c) => c.value === this.myChoice)?.emoji || '';
    const choiceText = new Text({
      text: `Your choice: ${myEmoji}`,
      style: { fontSize: 24, fill: COLORS.BLACK },
    });
    choiceText.x = 200 - choiceText.width / 2;
    choiceText.y = 150;
    this.container.addChild(choiceText);

    const wait = new Text({
      text: 'Waiting for round result...',
      style: { fontSize: 16, fill: COLORS.GRAY },
    });
    wait.x = 200 - wait.width / 2;
    wait.y = 200;
    this.container.addChild(wait);
  }

  private makeChoice(choice: Choice): void {
    if (this.gameState.status !== 'playing' || this.roundResult !== null) return;
    this.myChoice = choice;
    this.sendMove(choice);
    this.render();
  }

  private getStatusText(): string {
    if (this.gameState.status === 'waiting') return 'Waiting for players...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    if (this.myChoice) return 'Waiting for round result...';
    return `Round ${this.currentRound} - Score: ${this.scores.player} - ${this.scores.opponent}`;
  }
}
