import { MAPBOX_ACCESS_TOKEN, TIME_ZONE } from '$app/env/private';
import { fail, redirect } from '@sveltejs/kit';
import { parsePilot } from '#lib/pilots.js';
import { get, send, type FlightChoices, type SuggestionDetail } from '#lib/server/backend.js';
import { apiQuery, parse, query } from '../../logbook.js';

// The page's query is the suggestions list's, so its filters decide the
// suggestions and their order; `created` names a session just made.
export async function load({ fetch, params, url }) {
	const state = parse(url.searchParams);
	const [suggestion, choices] = await Promise.all([
		get<SuggestionDetail>(
			fetch,
			`/api/flights/suggestions/${params.key}?${apiQuery(state, TIME_ZONE)}`
		),
		get<FlightChoices>(fetch, '/api/flights/choices')
	]);
	return {
		suggestion,
		choices,
		state,
		created: url.searchParams.get('created'),
		// A public token: the browser needs it to load the map.
		mapboxToken: MAPBOX_ACCESS_TOKEN
	};
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
	/**
	 * Makes the flights a session, then on to the suggestion `then` (the
	 * flights left out, or the next one), which says the session was made.
	 */
	create: async ({ fetch, request, url }) => {
		const form = await request.formData();
		const body = { flight_ids: form.getAll('flight_id').map(String) };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		const session = result.data.id;

		const list = query(parse(url.searchParams));
		const then = String(form.get('then') ?? '');
		if (!then) redirect(303, `/flights${list}`);
		const params = new URLSearchParams(list);
		params.set('created', session);
		redirect(303, `/flights/suggestions/${then}?${params}`);
	}
};
