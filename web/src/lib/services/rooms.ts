import { api } from '$lib/api';
import type { Room, CreateRoomRequest, RoomMessage, CreateRoomMessageRequest } from '$lib/types';

export const roomService = {
  async list(gameId?: string): Promise<Room[]> {
    const params = gameId ? `?game_id=${gameId}` : '';
    return api.get<Room[]>(`/api/rooms${params}`, true);
  },

  async get(id: string): Promise<Room> {
    return api.get<Room>(`/api/rooms/${id}`, true);
  },

  async create(data: CreateRoomRequest): Promise<Room> {
    return api.post<Room, CreateRoomRequest>('/api/rooms', data, true);
  },

  async join(id: string): Promise<void> {
    return api.post<void>(`/api/rooms/${id}/join`, {}, true);
  },

  async leave(id: string): Promise<void> {
    return api.delete<void>(`/api/rooms/${id}/leave`, true);
  },

  async start(id: string): Promise<void> {
    return api.post<void>(`/api/rooms/${id}/start`, {}, true);
  },

  async getMessages(id: string, limit?: number): Promise<RoomMessage[]> {
    const params = limit ? `?limit=${limit}` : '';
    return api.get<RoomMessage[]>(`/api/rooms/${id}/messages${params}`, true);
  },

  async sendMessage(id: string, data: CreateRoomMessageRequest): Promise<RoomMessage> {
    return api.post<RoomMessage, CreateRoomMessageRequest>(`/api/rooms/${id}/messages`, data, true);
  }
};
