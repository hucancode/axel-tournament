<script lang="ts">
  import { onMount } from 'svelte';
  
  interface Props {
    gameCode: string;
    roomId: string;
    wsUrl: string;
  }
  
  let { gameCode, roomId, wsUrl }: Props = $props();
  
  let iframeRef: HTMLIFrameElement;
  let ws: WebSocket | null = null;
  
  onMount(() => {
    setupWebSocket();
    loadGameCode();
    
    return () => {
      ws?.close();
    };
  });
  
  function setupWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const apiHost = wsUrl.replace(/^https?:\/\//, '').replace(/^wss?:\/\//, '');
    ws = new WebSocket(`${protocol}//${apiHost}/ws/room/${roomId}`);
    
    ws.onopen = () => {
      console.log('Connected to game room');
    };
    
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      // Forward message to game iframe
      iframeRef?.contentWindow?.postMessage({
        type: 'game_message',
        data: message
      }, '*');
    };
    
    ws.onclose = () => {
      console.log('Disconnected from game room');
    };
  }
  
  function loadGameCode() {
    const gameAPI = `
      window.gameAPI = {
        sendMove: function(data) {
          window.parent.postMessage({
            type: 'game_move',
            data: data
          }, '*');
        },
        onMessage: function(callback) {
          window.addEventListener('message', function(event) {
            if (event.data.type === 'game_message') {
              callback(event.data.data);
            }
          });
        }
      };
    `;
    
    const fullGameCode = `
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="utf-8">
        <meta http-equiv="Content-Security-Policy" content="
          default-src 'self' 'unsafe-inline';
          script-src 'self' 'unsafe-inline';
          style-src 'self' 'unsafe-inline';
          img-src 'self' data: blob:;
          connect-src 'none';
          object-src 'none';
          frame-src 'none';
          form-action 'none';
          base-uri 'none';
        ">
        <title>Interactive Game</title>
      </head>
      <body>
        <script>${gameAPI}</script>
        ${gameCode}
      </body>
      </html>
    `;
    
    const blob = new Blob([fullGameCode], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    iframeRef.src = url;
  }
  
  // Handle messages from game iframe
  function handleMessage(event: MessageEvent) {
    if (event.source === iframeRef.contentWindow) {
      if (event.data.type === 'game_move') {
        ws?.send(JSON.stringify({
          type: 'game_move',
          data: event.data.data
        }));
      }
    }
  }
  
  onMount(() => {
    window.addEventListener('message', handleMessage);
    return () => window.removeEventListener('message', handleMessage);
  });
</script>

<div class="game-container">
  <iframe
    bind:this={iframeRef}
    title="Interactive Game"
    sandbox="allow-scripts allow-same-origin"
    class="game-iframe"
  ></iframe>
</div>

<style>
  .game-container {
    width: 100%;
    height: 600px;
    border: 1px solid #ccc;
    border-radius: 8px;
    overflow: hidden;
  }
  
  .game-iframe {
    width: 100%;
    height: 100%;
    border: none;
  }
</style>
