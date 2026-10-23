import { MAPBOX_ACCESS_TOKEN } from '$app/env/private';
import { fail } from '@sveltejs/kit';
import { parsePilot } from '#lib/pilots.js';
import {
	get,
	send,
	type Area,
	type FlightChoices,
	type FlightDetail,
	type FlightPage
} from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [flight, choices, areas] = await Promise.all([
		get<FlightPage>(fetch, `/api/flights/${params.id}`),
		get<FlightChoices>(fetch, '/api/flights/choices'),
		get<Area[]>(fetch, '/api/areas')
	]);
	// A public token: the browser needs it to load the map.
	return { flight, choices, areas, mapboxToken: MAPBOX_ACCESS_TOKEN };
}

export const actions = {
	/** Sets the pilot, as the logbook's batch action does for one flight. */
	setPilot: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = { flight_ids: [params.id], ...parsePilot(String(form.get('pilot') ?? '')) };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		const flight = await get<FlightDetail>(fetch, `/api/flights/${params.id}`);
		return { pilotName: flight.pilot_name };
	},
	/** Moves the flight to a group or the queue, as the logbook's batch action does. */
	setGroup: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const groupId = String(form.get('group_id') ?? '') || null;
		const body = { flight_ids: [params.id], group_id: groupId };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/group', body);
		if (!result.ok) return fail(result.status, { groupMessage: result.message });
		const flight = await get<FlightDetail>(fetch, `/api/flights/${params.id}`);
		return { groupName: flight.group_name };
	},
	/** Archives the flight, as the logbook's batch action does. */
	archive: async ({ fetch, params }) => {
		const body = { flight_ids: [params.id] };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/archive', body);
		if (!result.ok) return fail(result.status, { archiveMessage: result.message });
		return { archived: true };
	},
	restore: async ({ fetch, params }) => {
		const body = { flight_ids: [params.id] };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/restore', body);
		if (!result.ok) return fail(result.status, { restoreMessage: result.message });
		return { restored: true };
	}
};
