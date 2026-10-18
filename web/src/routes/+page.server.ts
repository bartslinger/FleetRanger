import { get, type FlightSummary } from '#lib/server/backend.js';

export async function load({ fetch }) {
	return { flights: await get<FlightSummary[]>(fetch, '/api/flights') };
}
