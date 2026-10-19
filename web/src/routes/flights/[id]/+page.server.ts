import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { fail } from '@sveltejs/kit';
import { parsePilot } from '#lib/pilots.js';
import { get, send, type FlightChoices, type FlightDetail } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [flight, choices] = await Promise.all([
		get<FlightDetail>(fetch, `/api/flights/${params.id}`),
		get<FlightChoices>(fetch, '/api/flights/choices')
	]);
	// A public token: the browser needs it to load the map.
	return { flight, choices, mapboxToken: MAPBOX_ACCESS_TOKEN };
}

export const actions = {
	/** Sets the pilot, as the logbook's batch action does for one flight. */
	setPilot: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = { flight_ids: [params.id], ...parsePilot(String(form.get('pilot') ?? '')) };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		// For the toast: an admin's change can move the flight to another team.
		const flight = await get<FlightDetail>(fetch, `/api/flights/${params.id}`);
		const moved = (flight.team_id ?? '') !== String(form.get('team_before') ?? '');
		return { pilotName: flight.pilot_name, movedTo: moved ? flight.team_name : null };
	},
	/** Moves the flight to a team or the queue, as the logbook's batch action does. Admins. */
	setTeam: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const teamId = String(form.get('team_id') ?? '') || null;
		const body = { flight_ids: [params.id], team_id: teamId };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/team', body);
		if (!result.ok) return fail(result.status, { teamMessage: result.message });
		const flight = await get<FlightDetail>(fetch, `/api/flights/${params.id}`);
		return { teamName: flight.team_name };
	}
};
