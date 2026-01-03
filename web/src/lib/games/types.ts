export interface GameProps {
  ws: WebSocket | null;
  wsConnected: boolean;
}

export type GameStatus = 'waiting' | 'playing' | 'finished';

export interface BaseGameState {
  status: GameStatus;
  result?: string;
}

export const COLORS = {
  WHITE: 0xffffff,
  BLACK: 0x000000,
  BLUE: 0x1976d2,
  RED: 0xf44336,
  GREEN: 0x4caf50,
  GRAY: 0x9e9e9e,
  LIGHT_GRAY: 0xf5f5f5,
} as const;

export function parseMessage(data: string): string[] {
  return data.trim().split(/\s+/);
}
