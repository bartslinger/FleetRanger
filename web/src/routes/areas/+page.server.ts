import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { error, fail } from '@sveltejs/kit';
import {
	get,
	remove,
	send,
	type Area,
	type FlightDetail,
	type FlightPage,
	type SavedArea
} from '#lib/server/backend.js';

export async function load({ fetch, locals }) {
	// Everyone sees the areas on maps; only admins manage them.
	if (locals.account?.role !== 'admin') error(403);
	const areas = await get<Area[]>(fetch, '/api/areas');
	// Without areas, the map starts where the latest flight was.
	let fallback: number[][] = [];
	if (areas.length === 0) {
		const latest = await get<FlightPage>(fetch, '/api/flights?size=1');
		if (latest.flights.length > 0) {
			const flight = await get<FlightDetail>(fetch, `/api/flights/${latest.flights[0].id}`);
			fallback = flight.track.coordinates;
		}
	}
	// A public token: the browser needs it to load the map.
	return { areas, fallback, mapboxToken: MAPBOX_ACCESS_TOKEN };
}

export const actions = {
	/** Creates an area, or changes the one with the given id. */
	save: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const name = String(form.get('name') ?? '');
		let boundary: unknown = null;
		try {
			boundary = JSON.parse(String(form.get('boundary') ?? 'null'));
		} catch {
			// The backend says what's missing.
		}
		const result = id
			? await send<SavedArea>(fetch, 'PUT', `/api/areas/${id}`, { name, boundary })
			: await send<SavedArea>(fetch, 'POST', '/api/areas', { name, boundary });
		if (!result.ok) {
			const nameTaken = result.status === 409;
			return fail(result.status, { [nameTaken ? 'nameMessage' : 'message']: result.message });
		}
		return { saved: result.data };
	},
	delete: async ({ fetch, request }) => {
		const id = String((await request.formData()).get('id') ?? '');
		const result = await remove(fetch, `/api/areas/${id}`);
		if (!result.ok) return fail(result.status, { deleteMessage: result.message });
		return { deleted: true };
	}
};
