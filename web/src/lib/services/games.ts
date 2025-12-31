import { api } from "../api";
import type { Game } from "../types";

export const gameService = {
  async list(): Promise<Game[]> {
    return api.get<Game[]>("/api/games");
  },
  async get(id: string): Promise<Game> {
    return api.get<Game>(`/api/games/${id}`);
  },
};
