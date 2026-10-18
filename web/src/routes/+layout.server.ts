import { TIME_ZONE } from '$app/env/private';
import { timeZoneName } from '#lib/format.js';

export function load({ locals }) {
	return {
		theme: locals.theme,
		timeZone: TIME_ZONE,
		// The abbreviation is computed here only: browsers and Node name zones
		// differently, and the page must render the same on both.
		timeZoneName: timeZoneName(TIME_ZONE)
	};
}
