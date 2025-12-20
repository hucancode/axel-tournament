import { api } from "../api";
import type {
  Submission,
  SubmissionResponse,
  CreateSubmissionRequest,
} from "../types";

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
};
