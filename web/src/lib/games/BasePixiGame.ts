import { Application, Container } from 'pixi.js';

export interface GameState {
  status: 'waiting' | 'playing' | 'finished';
  result?: string;
}

/** Per-game context passed when feeding v2 events to a PixiJS game. */
export interface GameContext {
  /** 0-based index of the local player in the room's `players` list. */
  myIndex: number;
  /** Room players in join order; index 0 is the first joiner. */
  players: string[];
}

export abstract class BasePixiGame {
  protected app!: Application;
  protected act: ((kind: string, payload: string) => void) | null;
  protected wsConnected: boolean;
  protected gameState: GameState = { status: 'waiting' };
  protected container: Container;
  protected ctx: GameContext = { myIndex: -1, players: [] };

  constructor(
    canvas: HTMLCanvasElement,
    act: ((kind: string, payload: string) => void) | null,
    wsConnected: boolean
  ) {
    this.act = act;
    this.wsConnected = wsConnected;
    this.container = new Container();
    this.initApp(canvas);
  }

  private async initApp(canvas: HTMLCanvasElement) {
    this.app = new Application();
    await this.app.init({ canvas, width: 400, height: 400, backgroundColor: 0xffffff });
    this.app.stage.addChild(this.container);
    this.render();
  }

  /** Apply a committed protocol event. State changes flow through here. */
  public abstract handleEvent(kind: string, payload: string, ctx: GameContext): void;

  protected abstract render(): void;

  protected sendMove(payload: string) {
    this.act?.('MOVE', payload);
  }

  public destroy() {
    this.app.destroy();
  }
}
