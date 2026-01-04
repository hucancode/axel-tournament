import { Graphics, Text } from 'pixi.js';
import { BasePixiGame } from './BasePixiGame';
import { COLORS, parseMessage } from './types';

type Choice = 'C' | 'D';

export class PrisonersDilemmaGame extends BasePixiGame {
  private myChoice: Choice | null = null;
  private opponentChoice: Choice | null = null;
  private scores = { player: 0, opponent: 0 };

  public handleMessage(data: string): void {
    const parts = parseMessage(data);
    if (!parts.length) return;

    switch (parts[0]) {
      case 'START':
        this.gameState.status = 'playing';
        this.render();
        break;
      case 'RESULT':
        if (parts.length === 5) {
          this.opponentChoice = parts[1] as Choice;
          this.myChoice = parts[2] as Choice;
          this.scores = { opponent: parseInt(parts[3]), player: parseInt(parts[4]) };
          this.render();
          setTimeout(() => {
            this.myChoice = null;
            this.opponentChoice = null;
            this.render();
          }, 2000);
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

    if (this.myChoice && this.opponentChoice) {
      this.renderResult();
    } else if (this.gameState.status === 'playing' && !this.myChoice) {
      this.renderChoices();
    }
  }

  private renderChoices(): void {
    // Cooperate button
    const coopButton = new Graphics();
    coopButton.roundRect(80, 150, 100, 60, 8);
    coopButton.fill(COLORS.GREEN);
    coopButton.interactive = true;
    coopButton.cursor = 'pointer';
    coopButton.on('pointerdown', () => this.makeChoice('C'));
    this.container.addChild(coopButton);

    const coopText = new Text({
      text: '🤝\nCooperate',
      style: { fontSize: 14, fill: COLORS.WHITE, align: 'center' }
    });
    coopText.x = 130 - coopText.width / 2;
    coopText.y = 165;
    this.container.addChild(coopText);

    // Defect button
    const defectButton = new Graphics();
    defectButton.roundRect(220, 150, 100, 60, 8);
    defectButton.fill(COLORS.RED);
    defectButton.interactive = true;
    defectButton.cursor = 'pointer';
    defectButton.on('pointerdown', () => this.makeChoice('D'));
    this.container.addChild(defectButton);

    const defectText = new Text({
      text: '⚔️\nDefect',
      style: { fontSize: 14, fill: COLORS.WHITE, align: 'center' }
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

    const resultText = new Text({
      text: `Round complete! Scores: You ${this.scores.player}, Opponent ${this.scores.opponent}`,
      style: { fontSize: 14, fill: COLORS.BLACK }
    });
    resultText.x = 200 - resultText.width / 2;
    resultText.y = 250;
    this.container.addChild(resultText);
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
    if (this.myChoice) return 'Waiting for round result...';
    return `Score: ${this.scores.player} - ${this.scores.opponent}`;
  }
}
