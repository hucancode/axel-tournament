import { BasePixiGame } from './BasePixiGame';
import { TicTacToeGame } from './TicTacToeGame';
import { RockPaperScissorsGame } from './RockPaperScissorsGame';
import { PrisonersDilemmaGame } from './PrisonersDilemmaGame';

export type GameConstructor = new (canvas: HTMLCanvasElement, sendMove: ((message: string) => void) | null, wsConnected: boolean) => BasePixiGame;

export const gameRegistry: Record<string, GameConstructor> = {
  'tic-tac-toe': TicTacToeGame,
  'rock-paper-scissors': RockPaperScissorsGame,
  'prisoners-dilemma': PrisonersDilemmaGame,
};

export function createGame(gameType: string, canvas: HTMLCanvasElement, sendMove: ((message: string) => void) | null, wsConnected: boolean): BasePixiGame | null {
  const GameClass = gameRegistry[gameType];
  return GameClass ? new GameClass(canvas, sendMove, wsConnected) : null;
}
