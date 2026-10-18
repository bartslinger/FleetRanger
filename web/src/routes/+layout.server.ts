import { TIME_ZONE } from '$app/env/private';
import { timeZoneName } from '#lib/format.js';
import { get, type ProcessingStatus } from '#lib/server/backend.js';

export async function load({ fetch, locals }) {
	return {
		theme: locals.theme,
		account: locals.account,
		timeZone: TIME_ZONE,
		// The abbreviation is computed here only: browsers and Node name zones
		// differently, and the page must render the same on both.
		timeZoneName: timeZoneName(TIME_ZONE),
		// Log processing progress, shown to admins in the header.
		processing:
			locals.account?.role === 'admin'
				? await get<ProcessingStatus>(fetch, '/api/logs/processing')
				: null
	};
}
