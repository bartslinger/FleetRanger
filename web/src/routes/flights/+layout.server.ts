import { isHttpError } from '@sveltejs/kit';
import { usesAreas } from '#lib/groups.js';
import {
	get,
	timeZone,
	type Area,
	type FlightNeighbours,
	type FlightPage
} from '#lib/server/backend.js';
import { apiFilters, parse } from './logbook.js';

/**
 * The flight open over the Flights cards (`?flight=<id>`), as a session is
 * over Sessions' (user's decision, 2026-11-18: Flights like Sessions), the
 * areas its map outlines, and the flights before and after it. Here and
 * not in the page's load, which reads only the list's own parameters:
 * opening, stepping through or closing a flight loads only this.
 */
export async function load({ fetch, locals, url }) {
	const id = url.searchParams.get('flight');
	if (!id) return { openFlight: null, openAreas: [] as Area[], flightNeighbours: null };
	// Stepping goes on into the weeks or months around, newest first, with
	// the list's other filters, so without its days.
	const state = parse(url.searchParams, timeZone(locals));
	const filters = apiFilters({ ...state, from: null, to: null }, timeZone(locals));
	filters.set('sort', '-takeoff_at');
	const [openFlight, openAreas, flightNeighbours] = await Promise.all([
		get<FlightPage>(fetch, `/api/flights/${encodeURIComponent(id)}`),
		usesAreas(locals.account) ? get<Area[]>(fetch, '/api/areas') : [],
		// None if the flight no longer passes the filters, e.g. its pilot changed.
		get<FlightNeighbours>(
			fetch,
			`/api/flights/${encodeURIComponent(id)}/neighbours?${filters}`
		).catch((failure) => {
			if (isHttpError(failure, 404)) return null;
			throw failure;
		})
	]);
	return { openFlight, openAreas, flightNeighbours };
}
