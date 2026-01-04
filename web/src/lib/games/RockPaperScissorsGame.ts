import { Graphics, Text } from 'pixi.js';
import { BasePixiGame } from './BasePixiGame';
import { COLORS, parseMessage } from './types';

type Choice = 'ROCK' | 'PAPER' | 'SCISSORS';

export class RockPaperScissorsGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private opponentChoice: Choice | null = null;
  private roundResult: 'WIN' | 'LOSE' | 'DRAW' | null = null;
  private scores = { player: 0, opponent: 0 };
  private currentRound = 0;

  private choices = [
    { value: 'ROCK' as Choice, emoji: '🪨', x: 80 },
    { value: 'PAPER' as Choice, emoji: '📄', x: 200 },
    { value: 'SCISSORS' as Choice, emoji: '✂️', x: 320 }
  ];

  public handleMessage(data: string): void {
    const parts = parseMessage(data);
    if (!parts.length) return;

    switch (parts[0]) {
      case 'START':
        this.gameState.status = 'playing';
        this.render();
        break;
      case 'ROUND':
        if (parts.length === 5 && parts[1] && parts[2] === 'SCORE') {
          this.currentRound = parseInt(parts[1]);
          const newScores = { player: parseInt(parts[3]), opponent: parseInt(parts[4]) };

          // Determine round result based on score change
          if (newScores.player > this.scores.player) {
            this.roundResult = 'WIN';
          } else if (newScores.opponent > this.scores.opponent) {
            this.roundResult = 'LOSE';
          } else {
            this.roundResult = 'DRAW';
          }

          this.scores = newScores;

          // Show result briefly, then reset for next round
          this.render();
          setTimeout(() => {
            this.myChoice = null;
            this.opponentChoice = null;
            this.roundResult = null;
            this.render();
          }, 500);
        }
        break;
      case 'SCORE':
        if (parts.length === 2) {
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

    // Status
    const status = new Text({
      text: this.getStatusText(),
      style: { fontSize: 16, fill: COLORS.BLACK }
    });
    status.x = 200 - status.width / 2;
    status.y = 20;
    this.container.addChild(status);

    if (this.gameState.status === 'playing' && !this.myChoice) {
      this.renderChoices();
    } else if (this.myChoice && this.roundResult) {
      this.renderResult();
    } else if (this.myChoice) {
      this.renderWaitingState();
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

    const myText = new Text({ text: `Your choice: ${myEmoji}`, style: { fontSize: 32, fill: COLORS.BLACK } });
    myText.x = 200 - myText.width / 2;
    myText.y = 120;
    this.container.addChild(myText);

    if (this.roundResult) {
      const resultText = this.roundResult === 'WIN' ? 'You Win This Round!' :
                         this.roundResult === 'LOSE' ? 'You Lose This Round!' :
                         'Draw!';
      const resultColor = this.roundResult === 'WIN' ? COLORS.GREEN :
                          this.roundResult === 'LOSE' ? COLORS.RED :
                          COLORS.GRAY;

      const result = new Text({
        text: resultText,
        style: { fontSize: 24, fill: resultColor }
      });
      result.x = 200 - result.width / 2;
      result.y = 180;
      this.container.addChild(result);
    }

    // Show current scores
    const scoreText = new Text({
      text: `Score: ${this.scores.player} - ${this.scores.opponent}`,
      style: { fontSize: 18, fill: COLORS.BLACK }
    });
    scoreText.x = 200 - scoreText.width / 2;
    scoreText.y = 230;
    this.container.addChild(scoreText);
  }

  private renderWaitingState(): void {
    const myEmoji = this.choices.find(c => c.value === this.myChoice)?.emoji || '';
    
    const choiceText = new Text({
      text: `Your choice: ${myEmoji}`,
      style: { fontSize: 24, fill: COLORS.BLACK }
    });
    choiceText.x = 200 - choiceText.width / 2;
    choiceText.y = 150;
    this.container.addChild(choiceText);

    const waitText = new Text({
      text: 'Waiting for round result...',
      style: { fontSize: 16, fill: COLORS.GRAY }
    });
    waitText.x = 200 - waitText.width / 2;
    waitText.y = 200;
    this.container.addChild(waitText);
  }

  private makeChoice(choice: Choice): void {
    // Don't allow new choice if still showing result from previous round
    if (this.gameState.status !== 'playing' || this.roundResult !== null) {
      return;
    }

    this.myChoice = choice;
    this.sendMessage(choice);
    this.render();
  }

  private getStatusText(): string {
    if (this.gameState.status === 'waiting') return 'Waiting for players...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    if (this.myChoice) return 'Waiting for round result...';
    return `Round ${this.currentRound} - Score: ${this.scores.player} - ${this.scores.opponent}`;
  }
}
