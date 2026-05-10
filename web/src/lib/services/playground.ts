import { env } from "$env/dynamic/public";

const JUDGE_URL = env.PUBLIC_JUDGE_URL || "http://localhost:8081";

export interface PlaygroundStartResponse {
  room_id: string;
  game_id: string;
  bot_player_id: string;
}

/**
 * Protocol-learning sandbox. The judge spins up a fresh room and
 * attaches a built-in sample bot; the caller then opens a normal
 * WebSocket via `RoomSocket` and plays as a human against it.
 */
export const playgroundService = {
  async start(gameId: string): Promise<PlaygroundStartResponse> {
    const token =
      typeof window !== "undefined"
        ? localStorage.getItem("auth_token")
        : null;
    if (!token) {
      throw new Error("not authenticated");
    }
    const resp = await fetch(`${JUDGE_URL}/api/playground/start`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ game_id: gameId }),
    });
    if (!resp.ok) {
      throw new Error(`playground start failed: HTTP ${resp.status}`);
    }
    return (await resp.json()) as PlaygroundStartResponse;
  },
};
