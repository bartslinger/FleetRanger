import { error, redirect } from '@sveltejs/kit';
import { me, signupStatus } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	// Someone logged in can arrive here only with a result to read.
	const result = url.searchParams.has('waiting') || url.searchParams.has('error');
	if (!result && (await me(fetch))) redirect(303, '/');
	const status = await signupStatus(fetch);
	if (!status) error(404);

	const problem = url.searchParams.get('error');
	return {
		...status,
		error:
			(['invalid', 'failed', 'expired', 'unverified'] as const).find(
				(value) => value === problem
			) ?? null,
		// Back from Google while sign-up was full.
		waiting: url.searchParams.has('waiting')
	};
}
