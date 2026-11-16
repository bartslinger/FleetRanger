import { error } from '@sveltejs/kit';
import { BACKEND_URL } from '$app/env/private';
import { signupStatus } from '#lib/server/backend.js';

/** Signing up: first Google, then naming the organisation, logged in but in
 * no organisation yet (user's decision, 2026-11-16). */
export async function load({ fetch, locals, url }) {
	const status = await signupStatus(fetch);
	if (!status) error(404);

	// Logged in: the form, unless they are admin of an organisation already.
	const organisations = locals.account?.organisations ?? locals.person?.organisations ?? null;
	let adminOf: { id: string; name: string } | null = null;
	if (organisations) {
		const response = await fetch(`${BACKEND_URL}/api/signup/admin`);
		if (!response.ok) error(502, `The backend answered ${response.status}.`);
		adminOf = (await response.json()).admin_of;
	}

	const problem = url.searchParams.get('error');
	return {
		...status,
		loggedIn: organisations !== null,
		organisations: organisations ?? [],
		adminOf,
		error:
			(['invalid', 'failed', 'unverified'] as const).find((value) => value === problem) ?? null,
		// Back from joining the waiting list.
		waiting: url.searchParams.has('waiting')
	};
}
