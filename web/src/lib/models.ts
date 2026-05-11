// User types
export type UserRole = "admin" | "player";
export type OAuthProvider = "google";

export interface User {
  id: string;
  email: string;
  username: string;
  role: UserRole;
  location: string;
  oauth_provider?: OAuthProvider;
  is_banned: boolean;
  ban_reason?: string;
  created_at: string;
  updated_at: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface RegisterRequest {
  email: string;
  username: string;
  password: string;
  location?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

// Tournament types
export type TournamentStatus =
  | "scheduled"
  | "registration"
  | "generating"
  | "running"
  | "completed"
  | "cancelled";

export type MatchGenerationType =
  | "all_vs_all"
  | "round_robin"
  | "single_elimination"
  | "double_elimination";

export type TournamentKind = "bot" | "human";

export interface Tournament {
  id: string;
  game_id: string;
  name: string;
  description: string;
  status: TournamentStatus;
  min_players: number;
  max_players: number;
  start_time?: string;
  end_time?: string;
  match_generation_type: MatchGenerationType;
  kind: TournamentKind;
  config?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface TournamentParticipant {
  id: string;
  tournament_id: string;
  user_id: string;
  submission_id?: string;
  score: number;
  wins: number;
  losses: number;
  draws: number;
  elo?: number;
  rank?: number;
  joined_at: string;
  username?: string;
}

export interface CreateTournamentRequest {
  game_id: string;
  name: string;
  description: string;
  min_players: number;
  max_players: number;
  start_time?: string;
  end_time?: string;
  match_generation_type?: MatchGenerationType;
  kind?: TournamentKind;
  config?: Record<string, unknown>;
}

export interface UpdateConfigRequest {
  config: Record<string, unknown>;
}

export interface UpdateTournamentRequest {
  name?: string;
  description?: string;
  status?: TournamentStatus;
  start_time?: string;
  end_time?: string;
}

// Game types
export type ProgrammingLanguage = "rust" | "go" | "c";

export interface Game {
  id: string;
  name: string;
  description: string;
  supported_languages: ProgrammingLanguage[];
  rounds_per_match: number;
  repetitions: number;
  bot_timeout_ms: number;
  human_timeout_ms: number;
  cpu_limit: number;
  bot_turn_timeout_ms: number;
  human_turn_timeout_ms: number;
  memory_limit_mb: number;
}

// Submission types
export type SubmissionStatus = "pending" | "accepted" | "failed";

export interface Submission {
  id: string;
  user_id: string;
  tournament_id: string;
  game_id: string;
  language: ProgrammingLanguage;
  code: string;
  status: SubmissionStatus;
  error_message?: string;
  created_at: string;
}

export interface CreateSubmissionRequest {
  tournament_id: string;
  language: ProgrammingLanguage;
  code: string;
}

export interface SubmissionResponse {
  id: string;
  tournament_id: string;
  language: ProgrammingLanguage;
  status: SubmissionStatus;
  created_at: string;
}

export interface SubmissionStats {
  submission_id: string;
  matches_played: number;
  wins: number;
  losses: number;
  draws: number;
  total_score: number;
}

// Match types
export type MatchStatus =
  | "pending"
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface MatchParticipant {
  user_id?: string;
  submission_id?: string;
  score?: number;
  metadata?: Record<string, any>;
}

export interface Match {
  id: string;
  tournament_id?: string;
  game_id: string;
  status: MatchStatus;
  participants: MatchParticipant[];
  metadata?: Record<string, any> | null;
  room_id?: string;
  error_message?: string;
  /// Users at fault (runtime error, illegal move, disconnect timeout).
  /// Frontend uses this to badge the row e.g. "crashed: alice".
  faulted_user_ids: string[];
  round?: number | null;
  bracket?: string | null;
  bracket_position?: number | null;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
}

export interface CreateMatchRequest {
  tournament_id: string;
  game_id: string;
  participant_submission_ids: string[];
}

export interface UpdateMatchResultRequest {
  participants: {
    submission_id: string;
    score: number;
    metadata?: Record<string, any>;
  }[];
  metadata?: Record<string, any>;
}

// Leaderboard types
export interface LeaderboardEntry {
  rank: number;
  user_id: string;
  username: string;
  location: string;
  score: number;
  tournament_name: string;
  tournament_id: string;
}

// Room types
export type RoomStatus = "lobby" | "playing" | "finished" | "abandoned";

/// Free-form per-game configuration. Each game owns the keys it reads:
///   tic-tac-toe: { board_size: number, win_chain: number }
///   rock-paper-scissors / prisoners-dilemma: { rounds: number }
///   chess / xiangqi: { time_pool_minutes?: number,
///                      time_per_turn_seconds?: number,
///                      blind?: boolean }
/// The server never validates the shape; backend logic falls back to
/// defaults for missing or invalid fields.
export type GameConfig = Record<string, unknown>;

export interface Room {
  id: string;
  game_id: string;
  host_id: string;
  name: string;
  max_players: number;
  status: RoomStatus;
  players: string[];
  tournament_id?: string;
  allowed_user_ids: string[];
  is_ranked: boolean;
  winner_id?: string;
  human_timeout_ms?: number;
  config?: GameConfig;
  created_at: string;
  updated_at: string;
}

export interface CreateRoomRequest {
  game_id: string;
  name: string;
  max_players: number;
  tournament_id?: string;
  human_timeout_ms?: number;
  config?: GameConfig;
}

export interface MatchmakingRequest {
  tournament_id: string;
}

export interface UpdateRoomRequest {
  name?: string;
  max_players?: number;
  status?: RoomStatus;
  human_timeout_ms?: number;
}

export interface RoomMessage {
  id: string;
  room_id: string;
  user_id: string;
  message: string;
  created_at: string;
}

export interface CreateRoomMessageRequest {
  message: string;
}

// API Error type
export interface ApiError {
  error: string;
}
