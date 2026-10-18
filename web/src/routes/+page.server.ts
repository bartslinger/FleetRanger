import { fail, redirect } from '@sveltejs/kit';
import { get, send, type FlightChoices, type FlightPage } from '#lib/server/backend.js';
import { parse, query } from './logbook.js';

export async function load({ fetch, url }) {
	const state = parse(url.searchParams);
	const api = new URLSearchParams({
		page: String(state.page),
		size: String(state.size),
		sort: `${state.sort.descending ? '-' : ''}${state.sort.key}`
	});
	const [{ flights, total }, choices] = await Promise.all([
		get<FlightPage>(fetch, `/api/flights?${api}`),
		get<FlightChoices>(fetch, '/api/flights/choices')
	]);

	// Past the last page, e.g. after flights were removed: go to the last one.
	const pages = Math.max(1, Math.ceil(total / state.size));
	if (state.page > pages) redirect(307, `${url.pathname}${query({ ...state, page: pages })}`);

	return { flights, total, state, choices };
}

/** The flights of a batch action, sent as one hidden field per flight. */
function flightIds(form: FormData) {
	return form.getAll('flight_id').map(String);
}

export const actions = {
	setPilot: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = {
			flight_ids: flightIds(form),
			pilot_id: String(form.get('pilot_id') ?? '') || null
		};
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		return { changed: result.data.changed };
	},
	setTeam: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = {
			flight_ids: flightIds(form),
			team_id: String(form.get('team_id') ?? '') || null
		};
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/team', body);
		if (!result.ok) return fail(result.status, { teamMessage: result.message });
		return { changed: result.data.changed };
	}
};
