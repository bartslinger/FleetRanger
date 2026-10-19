import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { fail, redirect } from '@sveltejs/kit';
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
	/** Sets the pilot of all the session's flights, as the logbook's batch action does. */
	setPilot: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const { flights } = await get<Session>(fetch, `/api/sessions/${params.id}`);
		const body = {
			flight_ids: flights.map((flight) => flight.id),
			pilot_id: String(form.get('pilot_id') ?? '') || null
		};
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		// For the toast: an admin's change can move the flights to another team.
		const after = await get<Session>(fetch, `/api/sessions/${params.id}`);
		const moved = (after.flights[0].team_id ?? '') !== String(form.get('team_before') ?? '');
		return {
			pilotName: after.flights[0].pilot_name,
			movedTo: moved ? after.flights[0].team_name : null
		};
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
