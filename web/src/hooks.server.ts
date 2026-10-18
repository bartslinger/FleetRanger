import { BACKEND_URL } from '$app/env/private';
import { redirect } from '@sveltejs/kit';
import type { Handle, HandleFetch } from '@sveltejs/kit/hooks';
import { me } from '#lib/server/backend.js';
import { parseTheme, THEME_COOKIE } from '#lib/theme.js';

/** Pages shown without logging in. */
const PUBLIC_PATHS = new Set(['/login', '/download']);

export const handle: Handle = async ({ event, resolve }) => {
	const theme = parseTheme(event.cookies.get(THEME_COOKIE));
	event.locals.theme = theme;

	// Everything else needs a login: send the browser to the login page, which
	// brings it back here afterwards. Public pages still know who is logged in.
	event.locals.account = await me(event.fetch);
	if (!event.locals.account && !PUBLIC_PATHS.has(event.url.pathname)) {
		const returnTo = event.url.pathname + event.url.search;
		redirect(303, `/login?${new URLSearchParams({ return_to: returnTo })}`);
	}

	// Renders the user's theme choice on <html>, so the page never shows the
	// other theme first.
	return resolve(event, {
		transformPageChunk: ({ html }) =>
			html.replace('%fleetranger.theme%', theme === 'system' ? '' : `data-theme="${theme}"`)
	});
};

/** Calls to the backend act for the browser: they carry its cookies and address. */
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
	if (request.url.startsWith(`${BACKEND_URL}/`)) {
		const cookie = event.request.headers.get('cookie');
		if (cookie) request.headers.set('cookie', cookie);
		request.headers.set('x-forwarded-for', event.getClientAddress());
	}
	return fetch(request);
};
