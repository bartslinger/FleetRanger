import { redirect } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
import { get, type FlightPage } from '#lib/server/backend.js';

// Sessions are the start page; the root only leads there. Until the
// organisation's first flight, its admins start on Equipment instead, where
// pairing a controller gets the first flights in (user's decision,
// 2026-11-19). Only admins: they see every flight, so none means the
// organisation has none, archived ones counted too.
export async function load({ fetch, locals }) {
	if (may(locals.account, 'controllers.pair', null) && !(await hasFlights(fetch))) {
		redirect(307, '/equipment');
	}
	redirect(307, '/sessions');
}

async function hasFlights(fetch: typeof globalThis.fetch): Promise<boolean> {
	try {
		const active = await get<FlightPage>(fetch, '/api/flights?size=1');
		if (active.total > 0) return true;
		const archived = await get<FlightPage>(fetch, '/api/flights?size=1&archived=true');
		return archived.total > 0;
	} catch {
		// The start page still opens; Sessions shows what went wrong.
		return true;
	}
}
