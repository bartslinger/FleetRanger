import { redirect } from '@sveltejs/kit';
import { signupStatus } from '#lib/server/backend.js';

/** After logging in with access to several organisations: the person picks
 * one (user's request, 2026-11-16), then goes where they were going. Their
 * invitations come first (user, 2026-11-17). */
export async function load({ fetch, locals, url }) {
	// Before choosing, the login is in no organisation; once in one, the page
	// only makes sense with more to choose from, or invitations.
	const who = locals.person ?? locals.account;
	const organisations = who?.organisations ?? [];
	const invitations = who?.invitations ?? [];
	if (!locals.person && organisations.length < 2 && invitations.length === 0) redirect(303, '/');
	// Logged in to sign up, with no organisation yet: back to naming one.
	if (organisations.length === 0 && invitations.length === 0 && (await signupStatus(fetch))) {
		redirect(303, '/signup');
	}
	// The backend only goes to paths on this site, whatever is passed here.
	return {
		organisations,
		invitations,
		returnTo: url.searchParams.get('return_to') ?? '/'
	};
}
