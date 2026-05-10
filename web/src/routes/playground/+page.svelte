<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { env } from "$env/dynamic/public";
  import { playgroundService } from "$services/playground";
  import { gameService } from "$services/games";
  import { authStore } from "$lib/stores/auth";
  import { Alert, LinkButton, PageHeader } from "$components";
  import type { Game } from "$lib/models";

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
  });

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
      logFrame("you", "send", `HELLO <jwt> 0`);
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
      logFrame("you", "recv", "PING");
      ws?.send("PONG");
      logFrame("you", "send", "PONG");
      return;
    }

    if (verb === "WELCOME") {
      const [pid] = rest.split(" ");
      humanPid = pid ?? null;
      connected = true;
      logFrame("you", "recv", text);
      // Server broadcasts events to every connection in the room, so the
      // bot also gets a WELCOME on its own (in-process) attach.
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

      if (kind === "GAME_STARTED") phase = "playing";
      if (kind === "GAME_END" || kind === "WINNER" || kind === "DRAW") {
        phase = "finished";
      }
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
    roomId = null;
    humanPid = null;
    botPid = null;
    connected = false;
    phase = "idle";
    frames = [];
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
        <p class="protocol-help">
          Your transport here is WebSocket (HELLO / WELCOME / PING / PONG visible).
          The submitted bot transport is stdio: drop HELLO, WELCOME, PING, PONG and
          everything else (ACT, EVENT, ERR) is identical. See
          <a href="https://github.com/anthropics/claude-code" onclick={(e) => e.preventDefault()}>protocols/wire.md</a>
          in the repo for full spec.
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
