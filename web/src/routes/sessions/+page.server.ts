import { MAPBOX_ACCESS_TOKEN, TIME_ZONE } from '$app/env/private';
import { fail, redirect } from '@sveltejs/kit';
import {
	get,
	send,
	type Devices,
	type FlightChoices,
	type SessionPage
} from '#lib/server/backend.js';
import { apiFilters, parse, query } from './list.js';

export async function load({ fetch, url }) {
	const state = parse(url.searchParams);
	const filters = apiFilters(state, TIME_ZONE);
	const api = new URLSearchParams(filters);
	api.set('page', String(state.page));
	api.set('size', String(state.size));
	// The filters' options: the equipment and teams of flights in a session.
	const options = new URLSearchParams(filters);
	options.set('session', 'any');
	const [{ sessions, total }, choices, devices] = await Promise.all([
		get<SessionPage>(fetch, `/api/sessions?${api}`),
		get<FlightChoices>(fetch, '/api/flights/choices'),
		get<Devices>(fetch, `/api/flights/devices?${options}`)
	]);

	// Past the last page, e.g. after sessions were merged: go to the last one.
	const pages = Math.max(1, Math.ceil(total / state.size));
	if (state.page > pages) redirect(307, `${url.pathname}${query({ ...state, page: pages })}`);

	return { sessions, total, state, choices, devices, mapboxToken: MAPBOX_ACCESS_TOKEN };
}

export const actions = {
	merge: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { session_ids: form.getAll('session_id').map(String) };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions/merge', body);
		if (!result.ok) return fail(result.status, { mergeMessage: result.message });
		return { merged: result.data.id };
	}
};
