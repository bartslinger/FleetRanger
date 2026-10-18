import { BACKEND_URL } from '$app/env/private';
import { error } from '@sveltejs/kit';

/** A flight as listed by `GET /api/flights`. */
export interface FlightSummary {
	id: string;
	takeoff_at: string;
	landed_at: string;
	air_time_s: number;
	distance_m: number;
	aircraft_serial: string | null;
	rc_serial: string | null;
}

/** One page of `GET /api/flights`. */
export interface FlightPage {
	flights: FlightSummary[];
	total: number;
}

/** Fetches JSON from the backend, failing the request if the backend does. */
export async function get<T>(fetch: typeof globalThis.fetch, path: string): Promise<T> {
	let response: Response;
	try {
		response = await fetch(`${BACKEND_URL}${path}`);
	} catch {
		error(503, 'The backend is not reachable.');
	}
	if (!response.ok) error(502, `The backend answered ${response.status}.`);
	return response.json();
}
