import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { fail, redirect } from '@sveltejs/kit';
import { parsePilot } from '#lib/pilots.js';
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
	/** Sets the pilot of these flights, as the logbook's batch action does. */
	setPilot: async ({ fetch, request }) => {
		const form = await request.formData();
		const pilot = parsePilot(String(form.get('pilot') ?? ''));
		const body = { flight_ids: form.getAll('flight_id').map(String), ...pilot };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		return { pilotId: pilot.pilot_id, changed: result.data.changed };
	},
	split: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { flight_ids: form.getAll('flight_id').map(String) };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions/split', body);
		if (!result.ok) return fail(result.status, { splitMessage: result.message });
		return { split: result.data.id };
	},
	remove: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { flight_ids: form.getAll('flight_id').map(String) };
		const result = await send<{ removed: number }>(fetch, 'POST', '/api/sessions/remove', body);
		if (!result.ok) return fail(result.status, { removeMessage: result.message });
		// All its flights gone, the session is too: back to the list.
		if (form.get('all') === 'true') redirect(303, '/sessions');
		return { removed: result.data.removed };
	}
};
