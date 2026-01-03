import { Graphics, Text } from 'pixi.js';
import { BasePixiGame } from './BasePixiGame';
import { COLORS, parseMessage } from './types';

type Choice = 'ROCK' | 'PAPER' | 'SCISSORS';

export class RockPaperScissorsGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private opponentChoice: Choice | null = null;
  private scores = { player: 0, opponent: 0 };
  private roundResult: string | null = null;

  private choices = [
    { value: 'ROCK' as Choice, emoji: '🪨', x: 80 },
    { value: 'PAPER' as Choice, emoji: '📄', x: 200 },
    { value: 'SCISSORS' as Choice, emoji: '✂️', x: 320 }
  ];

  protected handleMessage(data: string): void {
    const parts = parseMessage(data);
    if (!parts.length) return;

    switch (parts[0]) {
      case 'START':
        this.gameState.status = 'playing';
        this.render();
        break;
      case 'RESULT':
        if (parts.length === 4) {
          this.myChoice = parts[1] as Choice;
          this.opponentChoice = parts[2] as Choice;
          this.roundResult = parts[3];
          this.render();
          setTimeout(() => {
            this.myChoice = null;
            this.opponentChoice = null;
            this.roundResult = null;
            this.render();
          }, 3000);
        }
        break;
      case 'SCORE':
        if (parts.length === 3) {
          this.scores = { player: parseInt(parts[1]), opponent: parseInt(parts[2]) };
          this.render();
        }
        break;
      case 'END':
        this.gameState.status = 'finished';
        this.gameState.result = `Final: ${this.scores.player} - ${this.scores.opponent}`;
        this.render();
        break;
    }
  }

  protected render(): void {
    this.container.removeChildren();

    // Status
    const status = new Text({
      text: this.getStatusText(),
      style: { fontSize: 16, fill: COLORS.BLACK }
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
    this.choices.forEach(choice => {
      const button = new Graphics();
      button.circle(choice.x, 200, 40);
      button.fill(COLORS.LIGHT_GRAY);
      button.stroke({ width: 2, color: COLORS.GRAY });
      button.interactive = true;
      button.cursor = 'pointer';
      button.on('pointerdown', () => this.makeChoice(choice.value));
      this.container.addChild(button);

      const text = new Text({
        text: choice.emoji,
        style: { fontSize: 32 }
      });
      text.x = choice.x - text.width / 2;
      text.y = 200 - text.height / 2;
      this.container.addChild(text);
    });
  }

  private renderResult(): void {
    const myEmoji = this.choices.find(c => c.value === this.myChoice)?.emoji || '';
    const oppEmoji = this.choices.find(c => c.value === this.opponentChoice)?.emoji || '';

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

    if (this.roundResult) {
      const result = new Text({
        text: this.roundResult === 'WIN' ? 'You Win!' : this.roundResult === 'LOSE' ? 'You Lose!' : 'Draw!',
        style: { fontSize: 20, fill: this.roundResult === 'WIN' ? COLORS.GREEN : this.roundResult === 'LOSE' ? COLORS.RED : COLORS.GRAY }
      });
      result.x = 200 - result.width / 2;
      result.y = 250;
      this.container.addChild(result);
    }
  }

  private makeChoice(choice: Choice): void {
    if (!this.wsConnected || this.gameState.status !== 'playing' || this.myChoice) return;
    
    this.myChoice = choice;
    this.sendMessage(choice);
    this.render();
  }

  private getStatusText(): string {
    if (this.gameState.status === 'waiting') return 'Waiting for players...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    if (this.myChoice) return 'Waiting for opponent...';
    return `Score: ${this.scores.player} - ${this.scores.opponent}`;
  }
}
