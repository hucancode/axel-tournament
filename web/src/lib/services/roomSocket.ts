import { env } from '$env/dynamic/public';

const JUDGE_URL = env.PUBLIC_JUDGE_URL || "ws://localhost:8081";

export class RoomSocket {
  private ws: WebSocket | null = null;
  private gameId: string;
  private roomId: string;
  private playerId: string | null = null;
  private eventHandlers: Map<string, (data: string) => void> = new Map();
  private connected = false;
  private authenticated = false;
  private authResolver: ((value: string) => void) | null = null;
  private authRejecter: ((reason: string) => void) | null = null;

  constructor(gameId: string, roomId: string) {
    this.gameId = gameId;
    this.roomId = roomId;
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const wsUrl = JUDGE_URL.replace('http://', 'ws://').replace('https://', 'wss://');
      const url = `${wsUrl}/ws/${this.gameId}/${this.roomId}`;

      this.ws = new WebSocket(url);

      this.ws.onopen = () => {
        this.connected = true;
        console.log('WebSocket connected to room', this.roomId);
        resolve();
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        reject(error);
      };

      this.ws.onclose = () => {
        this.connected = false;
        this.authenticated = false;
        console.log('WebSocket disconnected from room', this.roomId);
        this.emit('disconnect', '');
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(event.data);
      };
    });
  }

  async auth(token: string): Promise<string> {
    return new Promise((resolve, reject) => {
      if (!this.connected) {
        reject('WebSocket not connected');
        return;
      }

      this.authResolver = resolve;
      this.authRejecter = reject;
      this.send(`LOGIN ${token}`);
    });
  }

  startGame(): void {
    this.send('START');
  }

  chat(message: string): void {
    this.send(`CHAT ${message}`);
  }

  leave(): void {
    this.send('LEAVE');
  }

  sendMove(move: string): void {
    this.send(move);
  }

  on(event: string, handler: (data: string) => void): void {
    this.eventHandlers.set(event, handler);
  }

  getPlayerId(): string | null {
    return this.playerId;
  }

  isAuthenticated(): boolean {
    return this.authenticated;
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.connected = false;
    this.authenticated = false;
  }

  private send(message: string): void {
    if (!this.ws || !this.connected) {
      throw new Error('WebSocket not connected');
    }
    // Allow LOGIN command without authentication
    if (!message.startsWith('LOGIN ') && !this.authenticated) {
      throw new Error('WebSocket not authenticated');
    }
    this.ws.send(message);
  }

  private handleMessage(data: string): void {
    console.log('WebSocket message:', data);

    const parts = data.split(' ');
    const command = parts[0];

    switch (command) {
      case 'LOGIN_OK':
        this.authenticated = true;
        this.playerId = parts[1];
        if (this.authResolver) {
          this.authResolver(this.playerId);
          this.authResolver = null;
          this.authRejecter = null;
        }
        // Check if this is a reconnect
        if (parts[2] === 'RECONNECT') {
          this.emit('reconnect', this.playerId);
        } else {
          this.emit('authenticated', this.playerId);
        }
        break;

      case 'LOGIN_FAILED':
        this.authenticated = false;
        const reason = parts.slice(1).join(' ');
        if (this.authRejecter) {
          this.authRejecter(reason);
          this.authResolver = null;
          this.authRejecter = null;
        }
        this.emit('auth_failed', reason);
        break;

      case 'CONNECTED':
        this.emit('connected', '');
        break;

      case 'RECONNECT':
        this.emit('reconnect', '');
        break;

      case 'REPLAY_START':
        this.emit('replay_start', '');
        break;
        
      case 'REPLAY_END':
        this.emit('replay_end', '');
        break;
        
      case 'PLAYER_JOINED':
        this.emit('player_joined', { userId: parts[1], username: parts[2] });
        break;
        
      case 'PLAYER_LEFT':
        this.emit('player_left', { userId: parts[1], username: parts[2] });
        break;
        
      case 'HOST_CHANGED':
        this.emit('host_changed', { userId: parts[1], username: parts[2] });
        break;
        
      case 'ROOM_CLOSED':
        this.emit('room_closed', '');
        break;
        
      case 'GAME_STARTED':
        this.emit('game_started', '');
        break;
        
      case 'GAME_FINISHED':
        this.emit('game_finished', parts.slice(1).join(' '));
        break;
        
      case 'CHAT':
        this.emit('chat', { userId: parts[1], username: parts[2], message: parts.slice(3).join(' ') });
        break;
        
      case 'ERROR':
        this.emit('error', parts.slice(1).join(' '));
        break;
        
      // Game-specific messages
      case 'START':
        this.emit('game_start', parts.slice(1).join(' '));
        break;
        
      case 'BOARD':
        this.emit('board_update', parts[1]);
        break;
        
      case 'TURN':
        this.emit('turn_update', parts[1]);
        break;
        
      case 'YOUR_TURN':
        this.emit('your_turn', '');
        break;
        
      case 'SCORE':
        this.emit('score_update', parts[1]);
        break;
        
      case 'END':
        this.emit('game_end', '');
        break;
        
      default:
        // Forward unknown messages as raw data
        this.emit('message', data);
        break;
    }
  }

  private emit(event: string, data: any): void {
    const handler = this.eventHandlers.get(event);
    if (handler) {
      handler(data);
    }
  }
}
