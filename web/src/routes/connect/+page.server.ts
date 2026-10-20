import { redirect } from '@sveltejs/kit';
import { send, type AppRequest } from '#lib/server/backend.js';

/**
 * The app's authorization request, as it sent the browser here: the page's
 * query, without a form action's name ("/allow").
 */
function request(url: URL): Record<string, string> {
	return Object.fromEntries([...url.searchParams].filter(([key]) => !key.startsWith('/')));
}

export async function load({ fetch, url }) {
	// The backend checks who asks and for what; a bad request is shown, and the
	// app isn't sent back an error, as it may not be who it says it is.
	const result = await send<AppRequest>(fetch, 'POST', '/api/oauth/request', request(url));
	return result.ok
		? { request: result.data, problem: null }
		: { request: null, problem: result.message };
}

async function answer(fetch: typeof globalThis.fetch, url: URL, allow: boolean) {
	const body = { ...request(url), allow };
	const result = await send<{ redirect: string }>(fetch, 'POST', '/api/oauth/approve', body);
	if (!result.ok) return { problem: result.message };
	// Back to the app, with a code or the refusal: an address the app
	// registered, which the backend checked.
	redirect(303, result.data.redirect, { external: true });
}

export const actions = {
	allow: ({ fetch, url }) => answer(fetch, url, true),
	cancel: ({ fetch, url }) => answer(fetch, url, false)
};
