import { api } from "../api";
import type { User } from "../models";

export const adminService = {
  async listUsers(page: number = 1, limit: number = 50): Promise<User[]> {
    return api.get<User[]>(
      `/api/admin/users?page=${page}&limit=${limit}`,
      true,
    );
  },
  async banUser(id: string, reason: string): Promise<void> {
    return api.post(`/api/admin/users/${id}/ban`, { reason }, true);
  },
  async unbanUser(id: string): Promise<void> {
    return api.post(`/api/admin/users/${id}/unban`, undefined, true);
  },
};
