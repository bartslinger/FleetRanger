import { redirect } from '@sveltejs/kit';
import { me } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	if (await me(fetch)) redirect(303, '/');

	const returnTo = url.searchParams.get('return_to') ?? '/';
	const error = url.searchParams.get('error');
	return {
		// The backend only returns to paths on this site, whatever is passed here.
		loginHref: `/auth/login?${new URLSearchParams({ return_to: returnTo })}`,
		error: error === 'no-access' || error === 'failed' ? error : null,
		loggedOut: url.searchParams.has('logged-out')
	};
}
