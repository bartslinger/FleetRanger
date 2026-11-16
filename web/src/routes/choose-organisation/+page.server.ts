import { redirect } from '@sveltejs/kit';

/** After logging in with access to several organisations: the person picks
 * one (user's request, 2026-11-16), then goes where they were going. */
export async function load({ locals, url }) {
	const organisations = locals.account?.organisations ?? [];
	if (organisations.length < 2) redirect(303, '/');
	// The backend only goes to paths on this site, whatever is passed here.
	return { organisations, returnTo: url.searchParams.get('return_to') ?? '/' };
}
