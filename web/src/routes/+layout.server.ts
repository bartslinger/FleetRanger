import { TIME_ZONE } from '$app/env/private';
import { get, type ProcessingStatus } from '#lib/server/backend.js';

export async function load({ fetch, locals }) {
	return {
		theme: locals.theme,
		account: locals.account,
		timeZone: TIME_ZONE,
		// Log processing progress, shown to admins in the header.
		processing:
			locals.account?.role === 'admin'
				? await get<ProcessingStatus>(fetch, '/api/logs/processing')
				: null
	};
}
