import { api } from "../api";
import type {
  Game,
  CreateGameRequest,
  UpdateGameRequest,
  GameTemplate,
  CreateGameTemplateRequest,
  Tournament,
  CreateTournamentRequest,
} from "../types";

export const gameSetterService = {
  // Games
  async listMyGames(): Promise<Game[]> {
    return api.get<Game[]>("/api/game-setter/games", true);
  },

  async createGame(data: CreateGameRequest): Promise<Game> {
    return api.post<Game>("/api/game-setter/games", data, true);
  },

  async updateGame(id: string, data: UpdateGameRequest): Promise<Game> {
    return api.put<Game>(`/api/game-setter/games/${id}`, data, true);
  },

  async deleteGame(id: string): Promise<void> {
    return api.delete<void>(`/api/game-setter/games/${id}`, true);
  },

  // Templates
  async createTemplate(data: CreateGameTemplateRequest): Promise<GameTemplate> {
    return api.post<GameTemplate>("/api/game-setter/templates", data, true);
  },

  async getTemplate(gameId: string, language: string): Promise<GameTemplate> {
    return api.get<GameTemplate>(`/api/game-setter/games/${gameId}/template/${language}`, true);
  },

  async listTemplates(gameId: string): Promise<GameTemplate[]> {
    return api.get<GameTemplate[]>(`/api/game-setter/templates?game_id=${gameId}`, true);
  },

  async updateTemplate(gameId: string, language: string, templateCode: string): Promise<GameTemplate> {
    return api.put<GameTemplate>(
      `/api/game-setter/games/${gameId}/template/${language}`,
      { template_code: templateCode },
      true,
    );
  },

  // Tournaments
  async createTournament(data: CreateTournamentRequest): Promise<Tournament> {
    return api.post<Tournament>("/api/game-setter/tournaments", data, true);
  },

  async updateTournament(id: string, data: Partial<CreateTournamentRequest>): Promise<Tournament> {
    return api.patch<Tournament>(`/api/game-setter/tournaments/${id}`, data, true);
  },

  async startTournament(id: string): Promise<Tournament> {
    return api.post<Tournament>(`/api/game-setter/tournaments/${id}/start`, {}, true);
  },

};
