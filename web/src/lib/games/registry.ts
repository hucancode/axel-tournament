import { BasePixiGame } from './BasePixiGame';
import { TicTacToeGame } from './TicTacToeGame';
import { RockPaperScissorsGame } from './RockPaperScissorsGame';
import { PrisonersDilemmaGame } from './PrisonersDilemmaGame';

export type ActFn = (kind: string, payload: string) => void;

export type GameConstructor = new (
  canvas: HTMLCanvasElement,
  act: ActFn | null,
  wsConnected: boolean
) => BasePixiGame;

export const gameRegistry: Record<string, GameConstructor> = {
  'tic-tac-toe': TicTacToeGame,
  'rock-paper-scissors': RockPaperScissorsGame,
  'prisoners-dilemma': PrisonersDilemmaGame,
};

export function createGame(
  gameType: string,
  canvas: HTMLCanvasElement,
  act: ActFn | null,
  wsConnected: boolean
): BasePixiGame | null {
  const GameClass = gameRegistry[gameType];
  return GameClass ? new GameClass(canvas, act, wsConnected) : null;
}
