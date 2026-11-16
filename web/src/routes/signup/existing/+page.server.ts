import { redirect } from '@sveltejs/kit';
import { BACKEND_URL } from '$app/env/private';

/** Back from Google after signing up with an account that already has an
 * organisation: the person chooses whether to create this one, or opens one
 * of theirs. The choices come from the login cookie (user's decision,
 * 2026-11-16), so without a sign-up in progress there is nothing here. */
export async function load({ fetch, locals }) {
	if (!locals.account) redirect(303, '/');
	const response = await fetch(`${BACKEND_URL}/api/signup/existing`);
	// Without choices in the cookie: they expired after ten minutes.
	if (response.status === 404) redirect(303, '/signup?error=expired');
	if (!response.ok) redirect(303, '/signup?error=failed');
	const status: { organisation_name: string; admin_of: { id: string; name: string } | null } =
		await response.json();
	return {
		organisationName: status.organisation_name,
		adminOf: status.admin_of,
		organisations: locals.account.organisations
	};
}
