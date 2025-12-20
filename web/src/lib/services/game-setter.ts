import { api } from "../api";
import type {
  Game,
  CreateGameRequest,
  UploadDockerfileRequest,
  GameTemplate,
  CreateGameTemplateRequest,
  Tournament,
  CreateTournamentRequest,
  MatchPolicy,
  CreateMatchPolicyRequest,
} from "../types";

export const gameSetterService = {
  // Games
  async listMyGames(): Promise<Game[]> {
    return api.get<Game[]>("/api/game-setter/games", true);
  },

  async createGame(data: CreateGameRequest): Promise<Game> {
    return api.post<Game>("/api/game-setter/games", data, true);
  },

  async updateGame(id: string, data: Partial<CreateGameRequest>): Promise<Game> {
    return api.put<Game>(`/api/game-setter/games/${id}`, data, true);
  },

  async deleteGame(id: string): Promise<void> {
    return api.delete<void>(`/api/game-setter/games/${id}`, true);
  },

  async uploadDockerfile(gameId: string, data: UploadDockerfileRequest): Promise<{ path: string; message: string }> {
    return api.post<{ path: string; message: string }>(`/api/game-setter/games/${gameId}/dockerfile`, data, true);
  },

  async uploadGameCode(gameId: string, language: string, codeContent: string): Promise<{ message: string; file_path: string }> {
    return api.post<{ message: string; file_path: string }>(`/api/game-setter/games/${gameId}/game-code`, { language, code_content: codeContent }, true);
  },

  // Templates
  async createTemplate(data: CreateGameTemplateRequest): Promise<GameTemplate> {
    return api.post<GameTemplate>("/api/game-setter/templates", data, true);
  },

  async getTemplate(gameId: string, language: string): Promise<GameTemplate> {
    return api.get<GameTemplate>(`/api/game-setter/templates/${gameId}/${language}`, true);
  },

  async listTemplates(gameId: string): Promise<GameTemplate[]> {
    return api.get<GameTemplate[]>(`/api/game-setter/templates?game_id=${gameId}`, true);
  },

  async updateTemplate(gameId: string, language: string, templateCode: string): Promise<GameTemplate> {
    return api.put<GameTemplate>(`/api/game-setter/templates/${gameId}/${language}`, { template_code: templateCode }, true);
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

  // Match Policy
  async createMatchPolicy(data: CreateMatchPolicyRequest): Promise<MatchPolicy> {
    return api.post<MatchPolicy>(`/api/game-setter/tournaments/${data.tournament_id}/policy`, data, true);
  },

  async getMatchPolicy(tournamentId: string): Promise<MatchPolicy> {
    return api.get<MatchPolicy>(`/api/game-setter/tournaments/${tournamentId}/policy`, true);
  },

  async updateMatchPolicy(tournamentId: string, data: Partial<CreateMatchPolicyRequest>): Promise<MatchPolicy> {
    return api.put<MatchPolicy>(`/api/game-setter/tournaments/${tournamentId}/policy`, data, true);
  },
};
