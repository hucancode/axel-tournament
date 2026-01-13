import { env } from '$env/dynamic/public';
import type { Room, CreateRoomRequest, RoomMessage, CreateRoomMessageRequest, RoomStatus } from '$lib/models';

const JUDGE_URL = env.PUBLIC_JUDGE_URL || "http://localhost:8081";

class JudgeApiClient {
  private getHeaders(includeAuth: boolean = false): HeadersInit {
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    };
    if (includeAuth && typeof window !== "undefined") {
      const token = localStorage.getItem("auth_token");
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }
    }
    return headers;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const error = await response.text().catch(() => `HTTP ${response.status}: ${response.statusText}`);
      throw new Error(error);
    }
    const text = await response.text();
    if (!text) return {} as T;
    return JSON.parse(text) as T;
  }

  async get<T>(path: string): Promise<T> {
    const response = await fetch(`${JUDGE_URL}${path}`, {
      method: "GET",
      headers: this.getHeaders(true),
    });
    return this.handleResponse<T>(response);
  }

  async post<T, D = any>(path: string, data?: D): Promise<T> {
    const response = await fetch(`${JUDGE_URL}${path}`, {
      method: "POST",
      headers: this.getHeaders(true),
      body: data ? JSON.stringify(data) : undefined,
    });
    return this.handleResponse<T>(response);
  }

  async delete<T>(path: string): Promise<T> {
    const response = await fetch(`${JUDGE_URL}${path}`, {
      method: "DELETE",
      headers: this.getHeaders(true),
    });
    return this.handleResponse<T>(response);
  }
}

const judgeApi = new JudgeApiClient();

// Judge server types
interface JudgeCreateRoomRequest {
  name: string;
  game_id: string;
  host_id: string;
  host_username: string;
  human_timeout_ms?: number;
}


interface JudgeRoomResponse {
  id: string;
  name: string;
  game_id: string;
  max_players: number;
  status: string;
  host_id: string;
  host_username: string;
  players: Array<{
    id: string;
    username: string;
    connected: boolean;
  }>;
  reconnecting: boolean;
}

export const roomService = {
  async list(gameId?: string): Promise<Room[]> {
    const params = gameId ? `?game_id=${gameId}` : '';
    const judgeRooms = await judgeApi.get<any[]>(`/api/rooms${params}`);

    // Convert Judge response to Room format
    return judgeRooms.map(room => ({
      id: room.id,
      name: room.name,
      game_id: room.game_id,
      max_players: room.max_players,
      status: room.status as RoomStatus,
      host_id: room.host_username, // Use host_username as display
      players: room.current_players ? Array(room.current_players).fill('').map((_, i) => `player_${i}`) : [],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }));
  },

  async get(id: string): Promise<Room> {
    const judgeRoom = await judgeApi.get<JudgeRoomResponse>(`/api/rooms/${id}`);

    // Convert Judge response to Room format
    return {
      id: judgeRoom.id,
      name: judgeRoom.name,
      game_id: judgeRoom.game_id,
      max_players: judgeRoom.max_players,
      status: judgeRoom.status as RoomStatus,
      host_id: judgeRoom.host_id,
      players: judgeRoom.players.map(p => p.id),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
  },

  async create(data: CreateRoomRequest): Promise<Room> {
    const userInfo = this.getCurrentUser();

    const judgeRequest: JudgeCreateRoomRequest = {
      name: data.name,
      game_id: data.game_id,
      host_id: userInfo.id,
      host_username: userInfo.username,
      human_timeout_ms: data.human_timeout_ms,
    };

    const judgeRoom = await judgeApi.post<JudgeRoomResponse>('/api/rooms', judgeRequest);

    return {
      id: judgeRoom.id,
      name: judgeRoom.name,
      game_id: judgeRoom.game_id,
      max_players: judgeRoom.max_players,
      status: judgeRoom.status as RoomStatus,
      host_id: judgeRoom.host_id,
      players: judgeRoom.players.map(p => p.id),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
  },

  // Helper to get current user info from JWT token
  getCurrentUser(): { id: string; username: string } {
    if (typeof window === "undefined") {
      throw new Error("Cannot get user info on server side");
    }

    const token = localStorage.getItem("auth_token");
    if (!token) {
      throw new Error("No auth token found");
    }

    try {
      // Decode JWT payload (simple base64 decode, not verifying signature)
      const payload = JSON.parse(atob(token.split('.')[1]));
      return {
        id: payload.sub,
        username: payload.username || payload.sub, // Fallback to sub if no username
      };
    } catch (e) {
      throw new Error("Invalid auth token");
    }
  },

};
