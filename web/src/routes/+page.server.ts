import { redirect } from '@sveltejs/kit';
import { anywhere } from '#lib/permissions.js';
import { get, type FlightPage } from '#lib/server/backend.js';

// Sessions are the start page; the root only leads there. Until the
// organisation's first flight, someone who can pair a controller starts on
// Equipment instead, where pairing gets the first flights in (user's
// decision, 2026-11-19).
export async function load({ fetch, locals }) {
	if (anywhere(locals.account, 'controllers.pair')) {
		const { total } = await get<FlightPage>(fetch, '/api/flights?size=1');
		if (total === 0) redirect(307, '/equipment');
	}
	redirect(307, '/sessions');
}
