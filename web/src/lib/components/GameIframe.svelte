<script lang="ts">
  import { onMount } from 'svelte';
  import { createGameIframeHTML } from './templates/game-iframe-template.ts';

  interface Props {
    gameCode: string;
    roomId: string;
    wsUrl: string;
  }

  let { gameCode, roomId, wsUrl }: Props = $props();

  let iframeRef: HTMLIFrameElement;
  let ws: WebSocket | null = null;
  let reconnectAttempts = 0;
  let shouldReconnect = true;
  const MAX_RECONNECT_ATTEMPTS = 5;
  const RECONNECT_DELAY = 2000;

  onMount(() => {
    console.log('[GameIframe] Component mounted, connecting to:', wsUrl);
    setupWebSocket();
    loadGameCode();
    window.addEventListener('message', handleMessage);

    return () => {
      console.log('[GameIframe] Component unmounting, closing WebSocket');
      shouldReconnect = false;
      ws?.close();
      window.removeEventListener('message', handleMessage);
    };
  });

  function setupWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const apiHost = wsUrl.replace(/^https?:\/\//, '').replace(/^wss?:\/\//, '');
    ws = new WebSocket(`${protocol}//${apiHost}/ws/room/${roomId}`);

    ws.onopen = () => {
      console.log('Connected to game room');
      reconnectAttempts = 0;
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
      if (shouldReconnect && reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        setTimeout(() => {
          reconnectAttempts++;
          console.log(`Reconnecting... attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS}`);
          setupWebSocket();
        }, RECONNECT_DELAY);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
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

    const fullGameCode = createGameIframeHTML(gameAPI, gameCode);

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

</script>

<div class="w-full h-150 border-4 border-black overflow-hidden bg-white">
  <iframe
    bind:this={iframeRef}
    title="Interactive Game"
    sandbox="allow-scripts allow-same-origin"
    class="w-full h-full border-0"
  ></iframe>
</div>
