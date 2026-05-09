import { env } from '$env/dynamic/public';

const JUDGE_URL = env.PUBLIC_JUDGE_URL || 'ws://localhost:8081';

/** A committed event read off the wire. Spec: judge/protocols/wire.md. */
export interface RoomEvent {
  seq: number;
  kind: string;
  payload: string;
}

/** Server-emitted error code + message. */
export interface RoomError {
  code: string;
  msg: string;
}

type EventHandler = (e: RoomEvent) => void;
type ConnectedHandler = (playerId: string) => void;
type DisconnectHandler = () => void;
type ErrorHandler = (err: RoomError) => void;

export interface RoomSocketHandlers {
  onConnected?: ConnectedHandler;
  onDisconnect?: DisconnectHandler;
  onEvent?: EventHandler;
  onError?: ErrorHandler;
}

/**
 * Pure wire transport. Spec: judge/protocols/wire.md.
 *
 * Knows: HELLO/WELCOME/ACT/EVENT/PING/PONG/ERR + reconnect cursor.
 * Does NOT know: lobby phases, chat, joins, host, players, or any
 * per-game semantics. Callers fold the EVENT stream themselves.
 */
export class RoomSocket {
  private ws: WebSocket | null = null;
  private url: string;
  private token: string | null = null;

  private playerId: string | null = null;
  private lastSeq = 0;
  private connecting = false;
  private connected = false;
  private intentionalClose = false;
  private authFailed = false;
  private reconnectDelay = 500;
  private connectResolve: ((pid: string) => void) | null = null;
  private connectReject: ((err: Error) => void) | null = null;

  private h: RoomSocketHandlers = {};

  constructor(gameId: string, roomId: string) {
    const base = JUDGE_URL.replace(/^http(s?):\/\//, 'ws$1://');
    this.url = `${base}/ws/${gameId}/${encodeURIComponent(roomId)}`;
  }

  setHandlers(h: RoomSocketHandlers): void {
    this.h = h;
  }

  getPlayerId(): string | null {
    return this.playerId;
  }

  getLastSeq(): number {
    return this.lastSeq;
  }

  isConnected(): boolean {
    return this.connected;
  }

  /** Open the socket, send HELLO, resolve when WELCOME arrives. */
  async connect(token: string): Promise<string> {
    this.token = token;
    return this.dial();
  }

  disconnect(): void {
    this.intentionalClose = true;
    this.token = null;
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.connected = false;
  }

  /** Send an ACT frame. Payload is opaque to this layer. */
  act(kind: string, payload: string = ''): void {
    if (payload) {
      this.send(`ACT ${kind} ${payload}`);
    } else {
      this.send(`ACT ${kind}`);
    }
  }

  private dial(): Promise<string> {
    if (this.connecting) return Promise.reject(new Error('already connecting'));
    this.connecting = true;
    return new Promise((resolve, reject) => {
      this.connectResolve = resolve;
      this.connectReject = reject;
      this.intentionalClose = false;
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        if (!this.token) {
          reject(new Error('no token'));
          return;
        }
        this.send(`HELLO ${this.token} ${this.lastSeq}`);
      };

      this.ws.onmessage = (ev) => this.handleFrame(String(ev.data));

      this.ws.onclose = () => {
        const wasConnected = this.connected;
        this.connected = false;
        this.connecting = false;
        this.h.onDisconnect?.();
        if (this.connectReject && !wasConnected) {
          this.connectReject(new Error('socket closed before WELCOME'));
          this.connectReject = null;
          this.connectResolve = null;
        }
        if (!this.intentionalClose && !this.authFailed && this.token) {
          setTimeout(() => {
            this.dial().catch((e) => console.error('reconnect failed:', e));
          }, this.reconnectDelay);
          this.reconnectDelay = Math.min(this.reconnectDelay * 2, 10_000);
        }
      };

      this.ws.onerror = (e) => {
        console.error('WebSocket error:', e);
      };
    });
  }

  private send(line: string): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('drop send (socket not open):', line);
      return;
    }
    this.ws.send(line);
  }

  private handleFrame(line: string): void {
    const sp1 = line.indexOf(' ');
    const verb = sp1 < 0 ? line : line.slice(0, sp1);
    const rest = sp1 < 0 ? '' : line.slice(sp1 + 1);

    switch (verb) {
      case 'WELCOME': {
        const [pid, headStr] = rest.split(' ');
        this.playerId = pid;
        // head_seq comes for free here; callers infer "caught up" from
        // their own EVENT cursor, so we don't surface it.
        void headStr;
        this.connected = true;
        this.connecting = false;
        this.reconnectDelay = 500;
        this.h.onConnected?.(pid);
        if (this.connectResolve) {
          this.connectResolve(pid);
          this.connectResolve = null;
          this.connectReject = null;
        }
        break;
      }
      case 'EVENT': {
        const sp2 = rest.indexOf(' ');
        const seq = parseInt(rest.slice(0, sp2), 10);
        const r2 = rest.slice(sp2 + 1);
        const sp3 = r2.indexOf(' ');
        const kind = sp3 < 0 ? r2 : r2.slice(0, sp3);
        const payload = sp3 < 0 ? '' : r2.slice(sp3 + 1);
        this.lastSeq = seq;
        this.h.onEvent?.({ seq, kind, payload });
        break;
      }
      case 'ERR': {
        const sp = rest.indexOf(' ');
        const code = sp < 0 ? rest : rest.slice(0, sp);
        const msg = sp < 0 ? '' : rest.slice(sp + 1);
        if (code === 'AUTH') {
          this.authFailed = true;
        }
        this.h.onError?.({ code, msg });
        break;
      }
      case 'PING':
        this.send('PONG');
        break;
      default:
        console.warn('unknown server frame:', line);
    }
  }
}
