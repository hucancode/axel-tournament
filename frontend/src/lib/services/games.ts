import { api } from '../api';
import type { Game, CreateGameRequest } from '../types';

export const gameService = {
	async list(): Promise<Game[]> {
		return api.get<Game[]>('/api/games');
	},

	async get(id: string): Promise<Game> {
		return api.get<Game>(`/api/games/${id}`);
	},

	// Admin endpoints
	async create(data: CreateGameRequest): Promise<Game> {
		return api.post<Game, CreateGameRequest>('/api/admin/games', data, true);
	},

	async update(id: string, data: Partial<CreateGameRequest>): Promise<Game> {
		return api.put<Game>(`/api/admin/games/${id}`, data, true);
	},

	async delete(id: string): Promise<void> {
		return api.delete(`/api/admin/games/${id}`, true);
	},
};
