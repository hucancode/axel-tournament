export function createGameIframeHTML(gameAPI: string, gameCode: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self' 'unsafe-inline';script-src 'self' 'unsafe-inline';style-src 'self' 'unsafe-inline';img-src 'self' data: blob:;connect-src 'none';object-src 'none';frame-src 'none';form-action 'none';base-uri 'none';">
  <title>Interactive Game</title>
</head>
<body>
  <script>${gameAPI}</script>
  ${gameCode}
</body>
</html>`;
}
