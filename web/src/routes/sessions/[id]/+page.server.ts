import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { get, type Session } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const session = await get<Session>(fetch, `/api/sessions/${params.id}`);
	// A public token: the browser needs it to load the map.
	return { session, mapboxToken: MAPBOX_ACCESS_TOKEN };
}
