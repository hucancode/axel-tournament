import { api } from "../api";
import type {
  Tournament,
  TournamentParticipant,
  CreateTournamentRequest,
  UpdateTournamentRequest,
} from "../types";

export const tournamentService = {
  async list(status?: string): Promise<Tournament[]> {
    const query = status ? `?status=${status}` : "";
    return api.get<Tournament[]>(`/api/tournaments${query}`);
  },
  async get(id: string): Promise<Tournament> {
    return api.get<Tournament>(`/api/tournaments/${id}`);
  },
  async getParticipants(id: string): Promise<TournamentParticipant[]> {
    return api.get<TournamentParticipant[]>(
      `/api/tournaments/${id}/participants`,
    );
  },
  async join(id: string): Promise<void> {
    return api.post(`/api/tournaments/${id}/join`, undefined, true);
  },
  async leave(id: string): Promise<void> {
    return api.delete(`/api/tournaments/${id}/leave`, true);
  },
  // Admin endpoints
  async create(data: CreateTournamentRequest): Promise<Tournament> {
    return api.post<Tournament, CreateTournamentRequest>(
      "/api/admin/tournaments",
      data,
      true,
    );
  },
  async update(
    id: string,
    data: UpdateTournamentRequest,
  ): Promise<Tournament> {
    return api.patch<Tournament>(`/api/admin/tournaments/${id}`, data, true);
  },
  async start(id: string): Promise<Tournament> {
    return api.post<Tournament>(`/api/admin/tournaments/${id}/start`, {}, true);
  },
};
