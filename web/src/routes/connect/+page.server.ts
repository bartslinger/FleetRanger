import { redirect } from '@sveltejs/kit';
import { send, type AppRequest } from '#lib/server/backend.js';

/** The names of form actions in a query, such as "/allow". */
const actionNames = (url: URL) => [...url.searchParams.keys()].filter((key) => key.startsWith('/'));

/**
 * The app's authorization request, as it sent the browser here: the page's
 * query, without the form action's name.
 */
function request(url: URL): Record<string, string> {
	return Object.fromEntries([...url.searchParams].filter(([key]) => !key.startsWith('/')));
}

export async function load({ fetch, url }) {
	// An action's name in the app's own link would make a button do something
	// else (Cancel would run "/allow"): such a request is refused.
	if (actionNames(url).length > 0) {
		return { request: null, query: '', problem: 'Malformed request' };
	}
	// The backend checks who asks and for what; a bad request is shown, and the
	// app isn't sent back an error, as it may not be who it says it is.
	const query = request(url);
	const result = await send<AppRequest>(fetch, 'POST', '/api/oauth/request', query);
	return result.ok
		? { request: result.data, query: new URLSearchParams(query).toString(), problem: null }
		: { request: null, query: '', problem: result.message };
}

async function answer(fetch: typeof globalThis.fetch, url: URL, allow: boolean) {
	// Exactly the one action the button names.
	if (actionNames(url).length !== 1) return { problem: 'Malformed request' };
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
