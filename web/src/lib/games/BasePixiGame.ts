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

/** Tone of the headline status banner shown above the canvas. */
export type StatusTone = 'idle' | 'turn' | 'wait' | 'win' | 'lose' | 'draw';

export interface GameStatus {
  text: string;
  tone: StatusTone;
  detail?: string;
}

export interface GameResult {
  outcome: 'win' | 'lose' | 'draw';
  title: string;
  details: string[];
}

export abstract class BasePixiGame {
  protected app!: Application;
  protected act: ((kind: string, payload: string) => void) | null;
  protected wsConnected: boolean;
  protected gameState: GameState = { status: 'waiting' };
  protected container: Container;
  protected ctx: GameContext = { myIndex: -1, players: [] };
  protected onUpdate: () => void = () => {};

  constructor(
    canvas: HTMLCanvasElement,
    act: ((kind: string, payload: string) => void) | null,
    wsConnected: boolean,
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

  /** Headline status for the banner above the canvas. */
  public abstract getStatus(): GameStatus;

  /** Final outcome screen, or null while game is in progress. */
  public getResult(): GameResult | null {
    return null;
  }

  public setOnUpdate(cb: () => void) {
    this.onUpdate = cb;
    cb();
  }

  protected abstract render(): void;

  /** Re-render canvas and notify Svelte to refresh status/result. */
  protected refresh() {
    this.render();
    this.onUpdate();
  }

  protected sendMove(payload: string) {
    this.act?.('MOVE', payload);
  }

  public destroy() {
    this.app.destroy();
  }
}
