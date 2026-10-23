import { TIME_ZONE } from '$app/env/private';
import { may } from '#lib/permissions.js';
import { get, type ProcessingStatus } from '#lib/server/backend.js';

export async function load({ fetch, locals }) {
	return {
		theme: locals.theme,
		account: locals.account,
		timeZone: TIME_ZONE,
		// Log processing progress, shown in the header to whoever manages logs.
		processing: may(locals.account, 'logs.manage', null)
			? await get<ProcessingStatus>(fetch, '/api/logs/processing')
			: null
	};
}
