import { api } from '../api';
import type { AuthResponse, LoginRequest, RegisterRequest, User } from '../types';

export const authService = {
	async register(data: RegisterRequest): Promise<AuthResponse> {
		return api.post<AuthResponse, RegisterRequest>('/api/auth/register', data);
	},

	async login(data: LoginRequest): Promise<AuthResponse> {
		return api.post<AuthResponse, LoginRequest>('/api/auth/login', data);
	},

	async resetPassword(email: string): Promise<{ message: string }> {
		return api.post('/api/auth/reset-password', { email });
	},

	async confirmReset(token: string, new_password: string): Promise<{ message: string }> {
		return api.post('/api/auth/confirm-reset', { token, new_password });
	},

	async getGoogleAuthUrl(): Promise<{ url: string }> {
		return api.get('/api/auth/google');
	},

	async handleGoogleCallback(code: string, state: string): Promise<AuthResponse> {
		return api.get<AuthResponse>(`/api/auth/google/callback?code=${code}&state=${state}`);
	},

	async getProfile(): Promise<User> {
		return api.get<User>('/api/users/profile', true);
	},

	async updateLocation(location: string): Promise<User> {
		return api.patch<User>('/api/users/location', { location }, true);
	},
};
