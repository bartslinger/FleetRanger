import { redirect } from '@sveltejs/kit';
import { dayOf } from '#lib/period.js';
import { get, timeZone, type FlightPage } from '#lib/server/backend.js';

/**
 * A flight opens over the Flights cards of its month (user's decision,
 * 2026-11-18: one way to see a flight); this address, used by links from
 * sessions, equipment and processing, goes there. An archived flight opens
 * among the archived ones.
 */
export async function load({ fetch, locals, params }) {
	const flight = await get<FlightPage>(fetch, `/api/flights/${encodeURIComponent(params.id)}`);
	const query = new URLSearchParams({
		month: dayOf(flight.takeoff_at, timeZone(locals)).slice(0, 7),
		...(flight.archived_at ? { status: 'archived' } : {}),
		flight: flight.id
	});
	redirect(307, `/flights?${query}`);
}
