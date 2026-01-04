import { Application, Container, Graphics, Text } from 'pixi.js';

export interface GameState {
  status: 'waiting' | 'playing' | 'finished';
  result?: string;
}

export abstract class BasePixiGame {
  protected app!: Application;
  protected ws: WebSocket | null;
  protected wsConnected: boolean;
  protected gameState: GameState = { status: 'waiting' };
  protected container: Container;

  constructor(canvas: HTMLCanvasElement, ws: WebSocket | null, wsConnected: boolean) {
    this.ws = ws;
    this.wsConnected = wsConnected;
    this.container = new Container();
    
    this.initApp(canvas);
    this.setupWebSocket();
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

  private setupWebSocket() {
    if (this.ws) {
      this.ws.addEventListener('message', (event) => {
        this.handleMessage(event.data);
      });
    }
  }

  protected sendMessage(message: string) {
    if (this.ws && this.wsConnected) {
      this.ws.send(message);
    }
  }

  protected abstract handleMessage(data: string): void;
  protected abstract render(): void;

  public destroy() {
    this.app.destroy();
  }
}
