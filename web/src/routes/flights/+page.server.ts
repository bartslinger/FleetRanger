import { TIME_ZONE } from '$app/env/private';
import { parsePilot } from '#lib/pilots.js';
import { fail, redirect } from '@sveltejs/kit';
import {
	get,
	send,
	type Devices,
	type FlightChoices,
	type FlightPage,
	type SuggestionPage
} from '#lib/server/backend.js';
import { apiQuery, parse, query } from './logbook.js';

export async function load({ fetch, url }) {
	const state = parse(url.searchParams);
	const api = apiQuery(state, TIME_ZONE);
	// The suggestions page through suggestions; the list's first page still
	// counts the flights.
	const listApi = new URLSearchParams(api);
	if (state.suggestions) listApi.set('page', '1');
	const [{ flights, total }, suggestions, choices, devices] = await Promise.all([
		get<FlightPage>(fetch, `/api/flights?${listApi}`),
		state.suggestions ? get<SuggestionPage>(fetch, `/api/flights/suggestions?${api}`) : null,
		get<FlightChoices>(fetch, '/api/flights/choices'),
		// The filters' options, narrowed by the other filters.
		get<Devices>(fetch, `/api/flights/devices?${api}`)
	]);

	// Past the last page, e.g. after flights were removed: go to the last one.
	const pages = Math.max(1, Math.ceil((suggestions?.total ?? total) / state.size));
	if (state.page > pages) redirect(307, `${url.pathname}${query({ ...state, page: pages })}`);

	return { flights, total, suggestions, state, choices, devices };
}

/**
 * The flights of a batch action: one hidden field per selected flight, or
 * `all` with the number the user saw, for all flights matching the list's
 * filters (the page's URL). Those are looked up now; if their number changed
 * meanwhile, the action is refused, so it never touches flights the user
 * didn't see counted.
 */
async function flightIds(
	fetch: typeof globalThis.fetch,
	url: URL,
	form: FormData
): Promise<{ ids: string[] } | { message: string }> {
	if (form.get('all') !== 'true') return { ids: form.getAll('flight_id').map(String) };
	const { ids } = await get<{ ids: string[] }>(
		fetch,
		`/api/flights/ids?${apiQuery(parse(url.searchParams), TIME_ZONE)}`
	);
	const expected = Number(form.get('expected'));
	if (ids.length !== expected) {
		return {
			message: `The list changed: ${ids.length} flights match now, not ${expected}. Check them and try again.`
		};
	}
	return { ids };
}

export const actions = {
	setPilot: async ({ fetch, request, url }) => {
		const form = await request.formData();
		const flights = await flightIds(fetch, url, form);
		if ('message' in flights) return fail(409, { pilotMessage: flights.message });
		const body = { flight_ids: flights.ids, ...parsePilot(String(form.get('pilot') ?? '')) };
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/pilot', body);
		if (!result.ok) return fail(result.status, { pilotMessage: result.message });
		return { changed: result.data.changed };
	},
	setTeam: async ({ fetch, request, url }) => {
		const form = await request.formData();
		const flights = await flightIds(fetch, url, form);
		if ('message' in flights) return fail(409, { teamMessage: flights.message });
		const body = {
			flight_ids: flights.ids,
			team_id: String(form.get('team_id') ?? '') || null
		};
		const result = await send<{ changed: number }>(fetch, 'POST', '/api/flights/team', body);
		if (!result.ok) return fail(result.status, { teamMessage: result.message });
		return { changed: result.data.changed };
	},
	newSession: async ({ fetch, request, url }) => {
		const form = await request.formData();
		const flights = await flightIds(fetch, url, form);
		if ('message' in flights) return fail(409, { newSessionMessage: flights.message });
		const body = { flight_ids: flights.ids };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions', body);
		if (!result.ok) return fail(result.status, { newSessionMessage: result.message });
		return { session: result.data.id };
	},
	removeFromSession: async ({ fetch, request, url }) => {
		const form = await request.formData();
		const flights = await flightIds(fetch, url, form);
		if ('message' in flights) return fail(409, { removeMessage: flights.message });
		const body = { flight_ids: flights.ids };
		const result = await send<{ removed: number }>(fetch, 'POST', '/api/sessions/remove', body);
		if (!result.ok) return fail(result.status, { removeMessage: result.message });
		return { removed: result.data.removed };
	},
	addToSession: async ({ fetch, request, url }) => {
		const form = await request.formData();
		const session = String(form.get('session_id') ?? '');
		if (!session) return fail(400, { addMessage: 'Choose a session.' });
		const flights = await flightIds(fetch, url, form);
		if ('message' in flights) return fail(409, { addMessage: flights.message });
		const body = { flight_ids: flights.ids };
		const path = `/api/sessions/${encodeURIComponent(session)}/flights`;
		const result = await send<{ id: string }>(fetch, 'POST', path, body);
		if (!result.ok) return fail(result.status, { addMessage: result.message });
		return { session: result.data.id };
	}
};
