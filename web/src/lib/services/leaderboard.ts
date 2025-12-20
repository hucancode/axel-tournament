import { api } from "../api";
import type { LeaderboardEntry } from "../types";

export const leaderboardService = {
  async get(filters?: {
    tournament_id?: string;
    game_id?: string;
    limit?: number;
  }): Promise<LeaderboardEntry[]> {
    const params = new URLSearchParams();
    if (filters?.tournament_id)
      params.append("tournament_id", filters.tournament_id);
    if (filters?.game_id) params.append("game_id", filters.game_id);
    if (filters?.limit) params.append("limit", filters.limit.toString());
    const query = params.toString() ? `?${params.toString()}` : "";
    return api.get<LeaderboardEntry[]>(`/api/leaderboard${query}`);
  },
};
