// User types
export type UserRole = "admin" | "gamesetter" | "player";
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
  | "running"
  | "completed"
  | "cancelled";

export interface Tournament {
  id: string;
  game_id: string;
  name: string;
  description: string;
  status: TournamentStatus;
  min_players: number;
  max_players: number;
  current_players: number;
  start_time?: string;
  end_time?: string;
  created_at: string;
  updated_at: string;
}

export interface TournamentParticipant {
  id: string;
  tournament_id: string;
  user_id: string;
  submission_id?: string;
  score: number;
  rank?: number;
  joined_at: string;
  username?: string;
}

export interface CreateTournamentRequest {
  game_id: string;
  name: string;
  description: string;
  status: TournamentStatus;
  min_players: number;
  max_players: number;
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
  is_active: boolean;
  owner_id: string;
  game_code: string;
  game_language: ProgrammingLanguage;
  rounds_per_match: number;
  repetitions: number;
  timeout_ms: number;
  cpu_limit: number;
  turn_timeout_ms: number;
  memory_limit_mb: number;
  created_at: string;
  updated_at: string;
}

type GameEditableFields = Omit<Game, "id" | "owner_id" | "created_at" | "updated_at">;

export type CreateGameRequest = Omit<GameEditableFields, "is_active">;

export type UpdateGameRequest = Partial<GameEditableFields>;

// Game Template types
export interface GameTemplate {
  id: string;
  game_id: string;
  language: ProgrammingLanguage;
  template_code: string;
  created_at: string;
  updated_at: string;
}

export interface CreateGameTemplateRequest {
  game_id: string;
  language: string;
  template_code: string;
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

// Match types
export type MatchStatus =
  | "pending"
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface MatchParticipant {
  submission_id: string;
  score?: number;
  metadata?: Record<string, any>;
}

export interface Match {
  id: string;
  tournament_id: string;
  game_id: string;
  status: MatchStatus;
  participants: MatchParticipant[];
  metadata?: Record<string, any> | null;
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

// API Error type
export interface ApiError {
  error: string;
}
