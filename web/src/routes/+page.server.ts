import { redirect } from '@sveltejs/kit';

// Sessions are the start page; the root only leads there.
export function load() {
	redirect(307, '/sessions');
}
