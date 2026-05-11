import { env } from "$env/dynamic/public";
import { api } from "../api";
import type { Room, CreateRoomRequest, RoomStatus, UpdateConfigRequest } from "$lib/models";

const JUDGE_URL = env.PUBLIC_JUDGE_URL || "http://localhost:8081";

interface JudgeRoomMeta {
  id: string;
  game_id: string;
  phase: "lobby" | "playing" | "finished";
  host: string | null;
  players: string[];
  head: number;
}

const PHASE_TO_STATUS: Record<JudgeRoomMeta["phase"], RoomStatus> = {
  lobby: "lobby",
  playing: "playing",
  finished: "finished",
};

/**
 * Room service. Discovery reads the judge's `room_meta` index; CRUD
 * (create / join / leave / start) goes through the api which persists
 * ranked metadata + tournament binding. The judge's WebSocket still
 * owns live game state.
 */
export const roomService = {
  /// Discover open rooms via the judge's read-only meta index.
  async listLive(gameId?: string): Promise<Room[]> {
    const params = gameId ? `?game=${encodeURIComponent(gameId)}` : "";
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
      host_id: m.host ?? "",
      max_players: 2,
      status: PHASE_TO_STATUS[m.phase] ?? "lobby",
      players: m.players,
      allowed_user_ids: [],
      is_ranked: false,
      created_at: now,
      updated_at: now,
    }));
  },

  /// Authoritative room list from the api (with tournament + ranked
  /// metadata). Use this when the user is browsing tournament-bound
  /// rooms.
  async list(gameId?: string): Promise<Room[]> {
    const query = gameId ? `?game_id=${encodeURIComponent(gameId)}` : "";
    return api.get<Room[]>(`/api/rooms${query}`);
  },

  async get(id: string): Promise<Room> {
    return api.get<Room>(`/api/rooms/${encodeURIComponent(id)}`);
  },

  async updateConfig(id: string, config: Record<string, unknown>): Promise<Room> {
    return api.patch<Room, UpdateConfigRequest>(
      `/api/rooms/${encodeURIComponent(id)}/config`,
      { config },
      true,
    );
  },

  async create(data: CreateRoomRequest): Promise<Room> {
    return api.post<Room, CreateRoomRequest>("/api/rooms", data, true);
  },

  async join(id: string): Promise<Room> {
    return api.post<Room>(`/api/rooms/${id}/join`, undefined, true);
  },

  async leave(id: string): Promise<void> {
    await api.delete(`/api/rooms/${id}/leave`, true);
  },

  async start(id: string): Promise<Room> {
    return api.post<Room>(`/api/rooms/${id}/start`, undefined, true);
  },

  /// Add the current user to the tournament's ranked-match queue.
  /// Pairing into a ranked room runs server-side on the next tick.
  async enqueueRanked(tournamentId: string): Promise<void> {
    await api.post(
      "/api/matchmaking/enqueue",
      { tournament_id: tournamentId },
      true,
    );
  },

  async dequeueRanked(tournamentId: string): Promise<void> {
    await api.post(
      "/api/matchmaking/dequeue",
      { tournament_id: tournamentId },
      true,
    );
  },

  getCurrentUser(): { id: string; username: string } {
    if (typeof window === "undefined") {
      throw new Error("Cannot get user info on server side");
    }
    const token = localStorage.getItem("auth_token");
    if (!token) throw new Error("No auth token found");
    try {
      const payload = JSON.parse(atob(token.split(".")[1]));
      return { id: payload.sub, username: payload.username || payload.sub };
    } catch {
      throw new Error("Invalid auth token");
    }
  },
};
