import { Application, Container, Graphics, Text } from 'pixi.js';

export interface GameState {
  status: 'waiting' | 'playing' | 'finished';
  result?: string;
}

export abstract class BasePixiGame {
  protected app!: Application;
  protected sendMove: ((message: string) => void) | null;
  protected wsConnected: boolean;
  protected gameState: GameState = { status: 'waiting' };
  protected container: Container;

  constructor(canvas: HTMLCanvasElement, sendMove: ((message: string) => void) | null, wsConnected: boolean) {
    this.sendMove = sendMove;
    this.wsConnected = wsConnected;
    this.container = new Container();

    this.initApp(canvas);
  }

  private async initApp(canvas: HTMLCanvasElement) {
    this.app = new Application();
    await this.app.init({
      canvas,
      width: 400,
      height: 400,
      backgroundColor: 0xffffff,
    });

    this.app.stage.addChild(this.container);
    this.render();
  }

  protected sendMessage(message: string) {
    if (this.sendMove) {
      this.sendMove(message);
    } else {
      console.error('BasePixiGame: Cannot send message - sendMove callback not available');
    }
  }

  public abstract handleMessage(data: string): void;
  protected abstract render(): void;

  public destroy() {
    this.app.destroy();
  }
}
