import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { get, type FlightDetail } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const flight = await get<FlightDetail>(fetch, `/api/flights/${params.id}`);
	// A public token: the browser needs it to load the map.
	return { flight, mapboxToken: MAPBOX_ACCESS_TOKEN };
}
