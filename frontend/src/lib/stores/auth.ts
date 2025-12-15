import { writable } from 'svelte/store';
import type { User } from '../types';

interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	loading: boolean;
}

function createAuthStore() {
	const initialState: AuthState = {
		user: null,
		token: null,
		isAuthenticated: false,
		loading: true,
	};

	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,
		setAuth: (user: User, token: string) => {
			if (typeof window !== 'undefined') {
				localStorage.setItem('auth_token', token);
				localStorage.setItem('auth_user', JSON.stringify(user));
			}
			set({ user, token, isAuthenticated: true, loading: false });
		},
		logout: () => {
			if (typeof window !== 'undefined') {
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}
			set({ user: null, token: null, isAuthenticated: false, loading: false });
		},
		initialize: () => {
			if (typeof window !== 'undefined') {
				const token = localStorage.getItem('auth_token');
				const userStr = localStorage.getItem('auth_user');

				if (token && userStr) {
					try {
						const user = JSON.parse(userStr) as User;
						set({ user, token, isAuthenticated: true, loading: false });
						return;
					} catch (e) {
						console.error('Failed to parse user data:', e);
					}
				}
			}
			set({ user: null, token: null, isAuthenticated: false, loading: false });
		},
		setLoading: (loading: boolean) => {
			update(state => ({ ...state, loading }));
		},
	};
}

export const authStore = createAuthStore();
