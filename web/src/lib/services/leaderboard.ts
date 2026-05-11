import { api } from "../api";
import type { LeaderboardEntry } from "../models";

export const leaderboardService = {
  async get(tournamentId: string, limit?: number): Promise<LeaderboardEntry[]> {
    const query = limit ? `?limit=${limit}` : "";
    return api.get<LeaderboardEntry[]>(
      `/api/tournaments/${tournamentId}/leaderboard${query}`,
    );
  },
};
