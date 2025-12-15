import { env } from '$env/dynamic/public';
import type { ApiError } from './types';

const API_URL = env.PUBLIC_API_URL || 'http://localhost:8080';

class ApiClient {
	private getHeaders(includeAuth: boolean = false): HeadersInit {
		const headers: HeadersInit = {
			'Content-Type': 'application/json',
		};

		if (includeAuth && typeof window !== 'undefined') {
			const token = localStorage.getItem('auth_token');
			if (token) {
				headers['Authorization'] = `Bearer ${token}`;
			}
		}

		return headers;
	}

	private async handleResponse<T>(response: Response): Promise<T> {
		if (!response.ok) {
			const error: ApiError = await response.json().catch(() => ({
				error: `HTTP ${response.status}: ${response.statusText}`
			}));
			throw new Error(error.error);
		}

		const text = await response.text();
		if (!text) return {} as T;

		return JSON.parse(text) as T;
	}

	async get<T>(path: string, authenticated: boolean = false): Promise<T> {
		const response = await fetch(`${API_URL}${path}`, {
			method: 'GET',
			headers: this.getHeaders(authenticated),
		});
		return this.handleResponse<T>(response);
	}

	async post<T, D = any>(path: string, data?: D, authenticated: boolean = false): Promise<T> {
		const response = await fetch(`${API_URL}${path}`, {
			method: 'POST',
			headers: this.getHeaders(authenticated),
			body: data ? JSON.stringify(data) : undefined,
		});
		return this.handleResponse<T>(response);
	}

	async put<T, D = any>(path: string, data: D, authenticated: boolean = false): Promise<T> {
		const response = await fetch(`${API_URL}${path}`, {
			method: 'PUT',
			headers: this.getHeaders(authenticated),
			body: JSON.stringify(data),
		});
		return this.handleResponse<T>(response);
	}

	async patch<T, D = any>(path: string, data: D, authenticated: boolean = false): Promise<T> {
		const response = await fetch(`${API_URL}${path}`, {
			method: 'PATCH',
			headers: this.getHeaders(authenticated),
			body: JSON.stringify(data),
		});
		return this.handleResponse<T>(response);
	}

	async delete<T>(path: string, authenticated: boolean = false): Promise<T> {
		const response = await fetch(`${API_URL}${path}`, {
			method: 'DELETE',
			headers: this.getHeaders(authenticated),
		});
		return this.handleResponse<T>(response);
	}
}

export const api = new ApiClient();
