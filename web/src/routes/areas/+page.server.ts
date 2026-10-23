import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { error, fail } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
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
	// Everyone sees the areas on maps; only who holds areas.manage manages them.
	if (!may(locals.account, 'areas.manage', null)) error(403);
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

/** The drawn shape, as the page sends it. */
function boundary(form: FormData): unknown {
	try {
		return JSON.parse(String(form.get('boundary') ?? 'null'));
	} catch {
		return null;
	}
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { name: String(form.get('name') ?? ''), boundary: boundary(form) };
		const result = await send<SavedArea>(fetch, 'POST', '/api/areas', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: result.data };
	},
	/** A new shape for an area; clipping may split it into more areas. */
	reshape: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const body = { boundary: boundary(form) };
		const result = await send<SavedArea>(fetch, 'PUT', `/api/areas/${id}`, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: result.data };
	},
	rename: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const body = { name: String(form.get('name') ?? '') };
		const result = await send<null>(fetch, 'PUT', `/api/areas/${id}`, body);
		if (!result.ok) return fail(result.status, { renameMessage: result.message });
		return { renamed: true };
	},
	delete: async ({ fetch, request }) => {
		const id = String((await request.formData()).get('id') ?? '');
		const result = await remove(fetch, `/api/areas/${id}`);
		if (!result.ok) return fail(result.status, { deleteMessage: result.message });
		return { deleted: true };
	}
};
