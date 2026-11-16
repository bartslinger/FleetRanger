import { redirect } from '@sveltejs/kit';

/** After logging in with access to several organisations: the person picks
 * one (user's request, 2026-11-16), then goes where they were going. */
export async function load({ locals, url }) {
	// Before choosing, the login is in no organisation; once in one, the page
	// only makes sense with more to choose from.
	const organisations = locals.person?.organisations ?? locals.account?.organisations ?? [];
	if (!locals.person && organisations.length < 2) redirect(303, '/');
	// Logged in to sign up, with no organisation yet: back to naming one.
	if (organisations.length === 0) redirect(303, '/signup');
	// The backend only goes to paths on this site, whatever is passed here.
	return { organisations, returnTo: url.searchParams.get('return_to') ?? '/' };
}
