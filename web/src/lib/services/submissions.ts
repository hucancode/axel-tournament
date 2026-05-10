import { api } from "../api";
import type {
  Submission,
  SubmissionResponse,
  SubmissionStats,
  CreateSubmissionRequest,
} from "../models";

export const submissionService = {
  async create(data: CreateSubmissionRequest): Promise<SubmissionResponse> {
    return api.post<SubmissionResponse, CreateSubmissionRequest>(
      "/api/submissions",
      data,
      true,
    );
  },
  async list(tournamentId?: string): Promise<Submission[]> {
    const query = tournamentId ? `?tournament_id=${tournamentId}` : "";
    return api.get<Submission[]>(`/api/submissions${query}`, true);
  },
  async get(id: string): Promise<Submission> {
    return api.get<Submission>(`/api/submissions/${id}`, true);
  },
  /// Mark this submission as the user's active bot for its tournament.
  /// Multi-bot upload is supported; only one is selected at a time.
  async select(id: string): Promise<SubmissionResponse> {
    return api.post<SubmissionResponse>(
      `/api/submissions/${id}/select`,
      undefined,
      true,
    );
  },
  /// Per-bot win/loss/draw + total score across every match it played.
  async stats(id: string): Promise<SubmissionStats> {
    return api.get<SubmissionStats>(`/api/submissions/${id}/stats`, true);
  },
};
