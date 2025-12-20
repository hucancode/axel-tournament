import { api } from "../api";
import type {
  Match,
  CreateMatchRequest,
  UpdateMatchResultRequest,
} from "../types";

export const matchService = {
  async list(filters?: {
    tournament_id?: string;
    game_id?: string;
    user_id?: string;
  }): Promise<Match[]> {
    const params = new URLSearchParams();
    if (filters?.tournament_id)
      params.append("tournament_id", filters.tournament_id);
    if (filters?.game_id) params.append("game_id", filters.game_id);
    if (filters?.user_id) params.append("user_id", filters.user_id);
    const query = params.toString() ? `?${params.toString()}` : "";
    return api.get<Match[]>(`/api/matches${query}`, true);
  },
  async get(id: string): Promise<Match> {
    return api.get<Match>(`/api/matches/${id}`, true);
  },
  async create(data: CreateMatchRequest): Promise<Match> {
    return api.post<Match, CreateMatchRequest>("/api/matches", data, true);
  },
  async updateResult(
    id: string,
    data: UpdateMatchResultRequest,
  ): Promise<Match> {
    return api.put<Match>(`/api/matches/${id}/result`, data, true);
  },
};
