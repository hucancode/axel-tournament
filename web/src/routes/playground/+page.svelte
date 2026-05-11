<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { env } from "$env/dynamic/public";
  import { playgroundService } from "$services/playground";
  import { gameService } from "$services/games";
  import { authStore } from "$lib/stores/auth";
  import { Alert, LinkButton, PageHeader } from "$components";
  import type { Game } from "$lib/models";
  import { createGame } from "$lib/games/registry";
  import type { BasePixiGame, GameStatus, GameResult } from "$lib/games/BasePixiGame";

  type Side = "you" | "bot";
  type Direction = "send" | "recv";
  interface WireFrame {
    side: Side;
    direction: Direction;
    text: string;
    seq?: number;
    t: number;
  }

  const JUDGE_URL = env.PUBLIC_JUDGE_URL || "ws://localhost:8081";
  const gameId = $derived(page.url.searchParams.get("game") || "");

  let game = $state<Game | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let roomId = $state<string | null>(null);
  let humanPid = $state<string | null>(null);
  let botPid = $state<string | null>(null);

  let frames = $state<WireFrame[]>([]);
  let ws: WebSocket | null = null;
  let connected = $state(false);
  let phase = $state<"idle" | "lobby" | "playing" | "finished">("idle");
  let actInput = $state("");

  let players = $state<string[]>([]);
  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  let pixiGame: BasePixiGame | null = null;
  let gameStatus = $state<GameStatus | null>(null);
  let gameResult = $state<GameResult | null>(null);
  let resultDismissed = $state(false);

  const auth = $derived($authStore);

  onMount(async () => {
    if (!auth.isAuthenticated) {
      goto("/login");
      return;
    }
    if (!gameId) {
      error = "Missing ?game=<id>";
      loading = false;
      return;
    }
    try {
      game = await gameService.get(gameId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load game";
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    ws?.close();
    ws = null;
    pixiGame?.destroy();
    pixiGame = null;
  });

  function initPixi() {
    if (!canvas || pixiGame) return;
    pixiGame = createGame(
      gameId,
      canvas,
      (kind, payload) => sendAct(kind, payload),
      connected,
    );
    pixiGame?.setOnUpdate(() => {
      gameStatus = pixiGame?.getStatus() ?? null;
      gameResult = pixiGame?.getResult() ?? null;
    });
  }

  function feedPixi(kind: string, payload: string) {
    pixiGame?.handleEvent(kind, payload, {
      myIndex: humanPid ? players.indexOf(humanPid) : -1,
      players,
    });
  }

  function logFrame(side: Side, direction: Direction, text: string, seq?: number) {
    frames = [...frames, { side, direction, text, seq, t: Date.now() }];
  }

  async function startSession() {
    if (!gameId) return;
    error = null;
    frames = [];
    phase = "lobby";
    try {
      const resp = await playgroundService.start(gameId);
      roomId = resp.room_id;
      botPid = resp.bot_player_id;
      // The bot is now subscribed to room events; it will JOIN as soon
      // as it sees the human's PLAYER_JOINED. Open the WebSocket.
      await openSocket(resp.room_id);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to start session";
      phase = "idle";
    }
  }

  async function openSocket(rid: string): Promise<void> {
    const token = localStorage.getItem("auth_token");
    if (!token) {
      error = "No auth token";
      return;
    }
    const base = JUDGE_URL.replace(/^http(s?):\/\//, "ws$1://");
    const url = `${base}/ws/${gameId}/${encodeURIComponent(rid)}`;
    ws = new WebSocket(url);

    ws.onopen = () => {
      const helloFrame = `HELLO ${token} 0`;
      ws?.send(helloFrame);
    };

    ws.onmessage = (ev) => {
      const text = String(ev.data);
      handleServerFrame(text);
    };

    ws.onclose = () => {
      connected = false;
      logFrame("you", "recv", "[socket closed]");
    };

    ws.onerror = () => {
      error = "WebSocket error";
    };
  }

  function handleServerFrame(text: string) {
    const sp = text.indexOf(" ");
    const verb = sp < 0 ? text : text.slice(0, sp);
    const rest = sp < 0 ? "" : text.slice(sp + 1);

    if (verb === "PING") {
      ws?.send("PONG");
      return;
    }

    if (verb === "WELCOME") {
      const [pid] = rest.split(" ");
      humanPid = pid ?? null;
      connected = true;
      return;
    }

    if (verb === "EVENT") {
      // EVENT <seq> <kind> [payload]
      const sp2 = rest.indexOf(" ");
      const seqStr = sp2 < 0 ? rest : rest.slice(0, sp2);
      const r2 = sp2 < 0 ? "" : rest.slice(sp2 + 1);
      const sp3 = r2.indexOf(" ");
      const kind = sp3 < 0 ? r2 : r2.slice(0, sp3);
      const payload = sp3 < 0 ? "" : r2.slice(sp3 + 1);
      const seq = parseInt(seqStr, 10);

      // Both clients (you + bot) receive every EVENT — show it on both
      // sides. Bot's own ACTs are silent on the wire (validate-rejects
      // are silent too), but every accepted ACT manifests as an EVENT,
      // so the wire log here is faithful to what each peer actually saw.
      logFrame("you", "recv", text, seq);
      logFrame("bot", "recv", text, seq);

      // Reconstruct the bot's outgoing ACT for action-bearing events
      // it originated. validate() rejects illegal ACTs silently, so the
      // accepted ACT is implicit when an EVENT lands carrying its pid.
      reflectBotAct(kind, payload);

      if (kind === "PLAYER_JOINED") {
        if (!players.includes(payload)) players = [...players, payload];
      } else if (kind === "PLAYER_LEFT") {
        players = players.filter((p) => p !== payload);
      }

      if (kind === "GAME_STARTED") {
        phase = "playing";
        gameResult = null;
        resultDismissed = false;
        tick().then(() => {
          initPixi();
          feedPixi(kind, payload);
        });
        return;
      }
      if (kind === "GAME_END" || kind === "WINNER" || kind === "DRAW") {
        phase = "finished";
      }
      feedPixi(kind, payload);
      return;
    }

    if (verb === "ERR") {
      logFrame("you", "recv", text);
      return;
    }

    logFrame("you", "recv", text);
  }

  /** Mirror the bot's likely ACT: every accepted ACT becomes one or
   *  more EVENTs whose payload starts with the originator's pid. We
   *  surface that as a synthesised "bot → server: ACT ..." line so
   *  the trace shows traffic on both halves of the wire. */
  function reflectBotAct(kind: string, payload: string) {
    if (!botPid) return;

    if (kind === "PLAYER_JOINED" && payload.trim() === botPid) {
      logFrame("bot", "send", `ACT JOIN`);
      return;
    }
    if (kind === "MOVE") {
      const sp = payload.indexOf(" ");
      const pid = sp < 0 ? payload : payload.slice(0, sp);
      const rest = sp < 0 ? "" : payload.slice(sp + 1);
      if (pid === botPid) {
        logFrame("bot", "send", `ACT MOVE ${rest}`);
      }
    }
  }

  function sendAct(kind: string, payload?: string) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    const frame = payload ? `ACT ${kind} ${payload}` : `ACT ${kind}`;
    ws.send(frame);
    logFrame("you", "send", frame);
  }

  function sendRaw() {
    if (!actInput.trim() || !ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(actInput);
    logFrame("you", "send", actInput);
    actInput = "";
  }

  function reset() {
    ws?.close();
    ws = null;
    pixiGame?.destroy();
    pixiGame = null;
    roomId = null;
    humanPid = null;
    botPid = null;
    connected = false;
    phase = "idle";
    frames = [];
    players = [];
    gameStatus = null;
    gameResult = null;
    resultDismissed = false;
  }

  // Quick-action presets keyed by game.
  const moveButtons = $derived.by(() => {
    if (gameId === "rock-paper-scissors") {
      return [
        { label: "MOVE ROCK", kind: "MOVE", payload: "ROCK" },
        { label: "MOVE PAPER", kind: "MOVE", payload: "PAPER" },
        { label: "MOVE SCISSORS", kind: "MOVE", payload: "SCISSORS" },
      ];
    }
    if (gameId === "prisoners-dilemma") {
      return [
        { label: "MOVE C", kind: "MOVE", payload: "C" },
        { label: "MOVE D", kind: "MOVE", payload: "D" },
      ];
    }
    if (gameId === "tic-tac-toe") {
      const cells: { label: string; kind: string; payload: string }[] = [];
      for (let r = 0; r < 3; r++) {
        for (let c = 0; c < 3; c++) {
          cells.push({ label: `${r},${c}`, kind: "MOVE", payload: `${r} ${c}` });
        }
      }
      return cells;
    }
    if (gameId === "chess") {
      return [
        { label: "e2 e4", kind: "MOVE", payload: "e2 e4 -" },
        { label: "g1 f3", kind: "MOVE", payload: "g1 f3 -" },
        { label: "e7 e5", kind: "MOVE", payload: "e7 e5 -" },
      ];
    }
    if (gameId === "xiangqi") {
      return [
        { label: "b3 e3", kind: "MOVE", payload: "b3 e3" },
        { label: "h3 e3", kind: "MOVE", payload: "h3 e3" },
        { label: "b8 e8", kind: "MOVE", payload: "b8 e8" },
      ];
    }
    if (gameId === "poker") {
      return [
        { label: "FOLD", kind: "FOLD", payload: "" },
        { label: "CHECK", kind: "CHECK", payload: "" },
        { label: "CALL", kind: "CALL", payload: "" },
        { label: "BET 50", kind: "BET", payload: "50" },
        { label: "RAISE 100", kind: "RAISE", payload: "100" },
        { label: "ALLIN", kind: "ALLIN", payload: "" },
      ];
    }
    return [];
  });
</script>

<style>
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--spacing-4);
  }
  header.intro {
    margin-bottom: var(--spacing-4);
  }
  .meta-row {
    display: flex;
    gap: var(--spacing-4);
    flex-wrap: wrap;
    font-family: monospace;
    font-size: 0.85rem;
    color: var(--color-fg-muted);
    margin-bottom: var(--spacing-4);
  }
  .panes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-4);
    margin-bottom: var(--spacing-4);
  }
  .pane {
    background: var(--color-bg-light);
    border: 1px solid var(--color-border);
    padding: var(--spacing-3);
    height: 460px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 0.78rem;
  }
  .pane h3 {
    margin: 0 0 var(--spacing-2) 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-fg-muted);
  }
  .frame {
    padding: 2px 6px;
    border-left: 3px solid transparent;
    word-break: break-all;
    white-space: pre-wrap;
  }
  .frame.send {
    border-left-color: var(--color-primary);
  }
  .frame.recv {
    border-left-color: var(--color-fg-muted);
  }
  .frame .arrow {
    color: var(--color-fg-muted);
    margin-right: 6px;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-2);
    margin-bottom: var(--spacing-3);
  }
  .raw-input {
    display: flex;
    gap: var(--spacing-2);
  }
  .raw-input input {
    flex: 1;
    font-family: monospace;
  }
  .start-section {
    background: var(--color-bg-light);
    padding: var(--spacing-6);
    text-align: center;
  }
  .protocol-help {
    font-size: 0.85rem;
    color: var(--color-fg-muted);
    margin-top: var(--spacing-3);
  }
  .phase-badge {
    display: inline-block;
    padding: 2px 8px;
    background: var(--color-bg-popup);
    border-radius: 4px;
    text-transform: uppercase;
    font-size: 0.75rem;
    letter-spacing: 0.05em;
  }
  .visualizer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-3);
    margin-bottom: var(--spacing-4);
    padding: var(--spacing-4);
    background: var(--color-bg-light);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
  }
  .status-banner {
    width: 100%;
    max-width: 500px;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-lg);
    border: var(--border-thick) solid var(--color-border);
    background: var(--color-bg);
    text-align: center;
  }
  .status-banner[data-tone="turn"] {
    border-color: var(--color-warning);
    background: color-mix(in srgb, var(--color-warning) 12%, var(--color-bg));
  }
  .status-banner[data-tone="wait"] {
    border-color: var(--color-info);
    background: color-mix(in srgb, var(--color-info) 12%, var(--color-bg));
  }
  .status-banner[data-tone="win"] {
    border-color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 18%, var(--color-bg));
  }
  .status-banner[data-tone="lose"] {
    border-color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 18%, var(--color-bg));
  }
  .status-banner[data-tone="draw"] {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 14%, var(--color-bg));
  }
  .status-text {
    font-size: var(--font-size-xl);
    font-weight: 700;
    color: var(--color-fg);
  }
  .status-detail {
    margin-top: 0.2rem;
    font-size: var(--font-size-sm);
    color: var(--color-fg-dim);
  }
  .canvas-wrap {
    position: relative;
  }
  canvas {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    display: block;
  }
  .result-overlay {
    position: absolute;
    inset: 0;
    background: rgb(0 0 0 / 0.6);
    border-radius: var(--radius-lg);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .result-card {
    background: var(--color-bg);
    border: var(--border-thick) solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 1.25rem;
    min-width: 260px;
    text-align: center;
  }
  .result-overlay[data-outcome="win"] .result-card { border-color: var(--color-success); }
  .result-overlay[data-outcome="lose"] .result-card { border-color: var(--color-error); }
  .result-overlay[data-outcome="draw"] .result-card { border-color: var(--color-accent); }
  .result-icon { font-size: 2.5rem; line-height: 1; }
  .result-title {
    margin: 0.4rem 0 0.6rem;
    font-size: var(--font-size-2xl);
    font-weight: 700;
  }
  .result-overlay[data-outcome="win"] .result-title { color: var(--color-success); }
  .result-overlay[data-outcome="lose"] .result-title { color: var(--color-error); }
  .result-overlay[data-outcome="draw"] .result-title { color: var(--color-accent); }
  .result-details {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
  }
  .result-actions {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
  }
</style>

<main>
  <div class="container">
    <PageHeader title="Protocol Playground" />

    {#if loading}
      <p>Loading game…</p>
    {:else if error}
      <Alert message={error} />
    {:else if game}
      <header class="intro">
        <p>
          Game: <strong>{game.name}</strong> ({game.id}). Act as a bot and play against a
          built-in sample bot. Every wire frame between the server and each client is
          shown below — the same protocol your submitted bot will use over stdio.
        </p>
      </header>

      <div class="meta-row">
        <span>room: {roomId ?? "—"}</span>
        <span>you: {humanPid ?? "—"}</span>
        <span>bot: {botPid ?? "—"}</span>
        <span>phase: <span class="phase-badge">{phase}</span></span>
        <span>ws: {connected ? "🟢 live" : "🔴 offline"}</span>
      </div>

      {#if phase === "idle"}
        <section class="start-section">
          <p>Start a sandbox session — the judge will spin up a fresh room and attach a sample bot.</p>
          <button data-variant="primary" onclick={startSession}>Start playground</button>
        </section>
      {:else}
        <div class="controls">
          {#if phase === "lobby"}
            <button data-variant="primary" onclick={() => sendAct("JOIN")}>ACT JOIN</button>
            <button data-variant="primary" onclick={() => sendAct("START")}>ACT START</button>
          {/if}
          {#if phase === "playing"}
            {#each moveButtons as btn}
              <button data-variant="secondary" onclick={() => sendAct(btn.kind, btn.payload)}>
                {btn.label}
              </button>
            {/each}
          {/if}
          <button data-variant="secondary" onclick={() => sendAct("LEAVE")}>ACT LEAVE</button>
          <button data-variant="secondary" onclick={() => sendAct("CHAT", "hello bot")}>
            ACT CHAT hello bot
          </button>
          <button data-variant="ghost" onclick={reset}>Reset</button>
        </div>

        <div class="raw-input">
          <input
            type="text"
            bind:value={actInput}
            placeholder="Raw frame, e.g. ACT MOVE ROCK"
            onkeydown={(e) => e.key === "Enter" && sendRaw()}
          />
          <button data-variant="secondary" onclick={sendRaw}>Send</button>
        </div>

        {#if phase === "playing" || phase === "finished"}
          <section class="visualizer">
            {#if gameStatus}
              <div class="status-banner" data-tone={gameStatus.tone}>
                <div class="status-text">{gameStatus.text}</div>
                {#if gameStatus.detail}
                  <div class="status-detail">{gameStatus.detail}</div>
                {/if}
              </div>
            {/if}
            <div class="canvas-wrap">
              <canvas bind:this={canvas}></canvas>
              {#if phase === "finished" && gameResult && !resultDismissed}
                <div class="result-overlay" data-outcome={gameResult.outcome}>
                  <div class="result-card">
                    <div class="result-icon">
                      {gameResult.outcome === "win" ? "🏆" : gameResult.outcome === "lose" ? "💔" : "🤝"}
                    </div>
                    <h2 class="result-title">{gameResult.title}</h2>
                    <ul class="result-details">
                      {#each gameResult.details as line}
                        <li>{line}</li>
                      {/each}
                    </ul>
                    <div class="result-actions">
                      <button data-variant="secondary" onclick={() => (resultDismissed = true)}>
                        Close
                      </button>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </section>
        {/if}
      {/if}

      <div class="panes">
        <div class="pane">
          <h3>You ↔ server</h3>
          {#each frames.filter((f) => f.side === "you") as f}
            <div class="frame {f.direction}">
              <span class="arrow">{f.direction === "send" ? "C→S" : "S→C"}</span>
              {f.text}
            </div>
          {/each}
        </div>
        <div class="pane">
          <h3>Bot ↔ server</h3>
          {#each frames.filter((f) => f.side === "bot") as f}
            <div class="frame {f.direction}">
              <span class="arrow">{f.direction === "send" ? "B→S" : "S→B"}</span>
              {f.text}
            </div>
          {/each}
        </div>
      </div>

      <LinkButton href="/games" variant="secondary" label="Back to games" />
    {/if}
  </div>
</main>
