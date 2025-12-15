import { authStore } from '$lib/stores/auth';
import { browser } from '$app/environment';

export const ssr = false;

export function load() {
	if (browser) {
		authStore.initialize();
	}
}
