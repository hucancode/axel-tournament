import gameTemplate from './game-template.html?raw';

export function createGameIframeHTML(gameAPI: string, gameCode: string): string {
  return gameTemplate
    .replace('<script id="game-api"></script>', `<script>${gameAPI}</script>`)
    .replace('<script id="game-code"></script>', gameCode);
}
