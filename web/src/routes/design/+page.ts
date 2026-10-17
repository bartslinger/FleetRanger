import { dev } from '$app/env';
import { error } from '@sveltejs/kit';

// The component overview exists only on the dev server.
export function load() {
	if (!dev) error(404);
}
