import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { fail } from '@sveltejs/kit';
import { get, send, type FlightChoices, type Session } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [session, choices] = await Promise.all([
		get<Session>(fetch, `/api/sessions/${params.id}`),
		get<FlightChoices>(fetch, '/api/flights/choices')
	]);
	// A public token: the browser needs it to load the map.
	return { session, choices, mapboxToken: MAPBOX_ACCESS_TOKEN };
}

export const actions = {
	split: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { flight_ids: form.getAll('flight_id').map(String) };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions/split', body);
		if (!result.ok) return fail(result.status, { splitMessage: result.message });
		return { split: result.data.id };
	}
};
