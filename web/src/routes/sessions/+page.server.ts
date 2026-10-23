import { MAPBOX_ACCESS_TOKEN, TIME_ZONE } from '$app/env/private';
import { fail, redirect } from '@sveltejs/kit';
import {
	get,
	send,
	type Devices,
	type FlightChoices,
	type SessionPage
} from '#lib/server/backend.js';
import { parsePilot } from '#lib/pilots.js';
import { listFilters, parse, PERIOD_SIZE, query } from './list.js';

export async function load({ fetch, url }) {
	const state = parse(url.searchParams, TIME_ZONE);
	const filters = listFilters(state, TIME_ZONE);
	const api = new URLSearchParams(filters);
	// The cards show their whole period on one page.
	const cards = state.view === 'cards';
	api.set('page', cards ? '1' : String(state.page));
	api.set('size', cards ? String(PERIOD_SIZE) : String(state.size));
	// The cards' map shows all their tracks, coarse.
	if (cards) api.set('tracks', '1');
	// The filters' options: the equipment of flights in a session, and the
	// sessions' own groups.
	const options = new URLSearchParams(filters);
	options.set('session', 'any');
	const [{ sessions, total }, choices, devices] = await Promise.all([
		get<SessionPage>(fetch, `/api/sessions?${api}`),
		get<FlightChoices>(fetch, '/api/flights/choices'),
		get<Devices>(fetch, `/api/flights/devices?${options}`)
	]);

	// Past the last page, e.g. after sessions were merged: go to the last one.
	const pages = Math.max(1, Math.ceil(total / state.size));
	if (!cards && state.page > pages)
		redirect(307, `${url.pathname}${query({ ...state, page: pages })}`);

	return { sessions, total, state, choices, devices, mapboxToken: MAPBOX_ACCESS_TOKEN };
}

export const actions = {
	merge: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { session_ids: form.getAll('session_id').map(String) };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions/merge', body);
		if (!result.ok) return fail(result.status, { mergeMessage: result.message });
		return { merged: result.data.id };
	},
	// The open session's flights, as on its page before.
	/** Sets the pilot of these flights, as the logbook's batch action does. */
	setPilot: async ({ fetch, request }) => {
		const form = await request.formData();
		const pilot = parsePilot(String(form.get('pilot') ?? ''));
		const body = { flight_ids: form.getAll('flight_id').map(String), ...pilot };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		return { pilotId: pilot.pilot_id, changed: result.data.changed };
	},
	/** Moves the open session to another group, and its flights if asked. */
	setGroup: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('session_id') ?? '');
		const group = String(form.get('group_id') ?? '');
		const body = {
			group_id: group === '' ? null : group,
			move_flights: form.get('move_flights') === 'true'
		};
		const path = `/api/sessions/${encodeURIComponent(id)}/group`;
		const result = await send<{ id: string }>(fetch, 'POST', path, body);
		if (!result.ok) return fail(result.status, { groupMessage: result.message });
		return { moved: body.move_flights };
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
