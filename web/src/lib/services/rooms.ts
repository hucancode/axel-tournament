import { env } from '$env/dynamic/public';
import type { Room, CreateRoomRequest, RoomStatus } from '$lib/models';

const JUDGE_URL = env.PUBLIC_JUDGE_URL || 'http://localhost:8081';

interface JudgeRoomMeta {
  id: string;
  game_id: string;
  phase: 'lobby' | 'playing' | 'finished';
  host: string | null;
  players: string[];
  head: number;
}

const PHASE_TO_STATUS: Record<JudgeRoomMeta['phase'], RoomStatus> = {
  lobby: 'waiting',
  playing: 'playing',
  finished: 'finished',
};

/**
 * Client-side room service. Discovery reads the judge's `room_meta` index;
 * room IDs are allocated locally because the judge no longer has CRUD —
 * a room comes into existence on the first ACT (typically auto-JOIN).
 */
export const roomService = {
  async list(gameId?: string): Promise<Room[]> {
    const params = gameId ? `?game=${encodeURIComponent(gameId)}` : '';
    const resp = await fetch(`${JUDGE_URL}/api/rooms${params}`);
    if (!resp.ok) {
      throw new Error(`Failed to list rooms: HTTP ${resp.status}`);
    }
    const metas = (await resp.json()) as JudgeRoomMeta[];
    const now = new Date().toISOString();
    return metas.map((m) => ({
      id: m.id,
      name: m.id,
      game_id: m.game_id,
      host_id: m.host ?? '',
      max_players: 2,
      status: PHASE_TO_STATUS[m.phase] ?? 'waiting',
      players: m.players,
      created_at: now,
      updated_at: now,
    }));
  },

  /** Returns a synthetic Room shaped from the id alone. Real player/host/phase
   *  data is delivered by the WebSocket on connect (see RoomSocket). */
  async get(id: string): Promise<Room> {
    const now = new Date().toISOString();
    return {
      id,
      game_id: '',
      host_id: '',
      name: id,
      max_players: 2,
      status: 'waiting' as RoomStatus,
      players: [],
      created_at: now,
      updated_at: now,
    };
  },

  /** Allocate a room ID locally; server-side row is created on first ACT. */
  async create(data: CreateRoomRequest): Promise<Room> {
    const id = `room:${cryptoRandomId()}`;
    const user = this.getCurrentUser();
    const now = new Date().toISOString();
    return {
      id,
      game_id: data.game_id,
      host_id: user.id,
      name: data.name,
      max_players: data.max_players,
      status: 'waiting' as RoomStatus,
      players: [],
      human_timeout_ms: data.human_timeout_ms,
      created_at: now,
      updated_at: now,
    };
  },

  getCurrentUser(): { id: string; username: string } {
    if (typeof window === 'undefined') {
      throw new Error('Cannot get user info on server side');
    }
    const token = localStorage.getItem('auth_token');
    if (!token) throw new Error('No auth token found');
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return { id: payload.sub, username: payload.username || payload.sub };
    } catch {
      throw new Error('Invalid auth token');
    }
  },
};

function cryptoRandomId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID().replace(/-/g, '').slice(0, 16);
  }
  return Math.random().toString(36).slice(2, 18);
}
