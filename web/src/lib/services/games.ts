import { api } from "../api";
import type { Game, ProgrammingLanguage } from "../models";

export interface StarterCode {
  game_id: string;
  language: ProgrammingLanguage;
  filename: string;
  code: string;
}

export const gameService = {
  async list(): Promise<Game[]> {
    return api.get<Game[]>("/api/games");
  },
  async get(id: string): Promise<Game> {
    return api.get<Game>(`/api/games/${id}`);
  },
  async getStarter(id: string, language: ProgrammingLanguage): Promise<StarterCode> {
    return api.get<StarterCode>(`/api/games/${id}/starter/${language}`);
  },
};
