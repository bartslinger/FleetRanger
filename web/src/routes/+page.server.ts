import { redirect } from '@sveltejs/kit';
import { get, type FlightPage } from '#lib/server/backend.js';
import { parse, query } from './logbook.js';

export async function load({ fetch, url }) {
	const state = parse(url.searchParams);
	const api = new URLSearchParams({
		page: String(state.page),
		size: String(state.size),
		sort: `${state.sort.descending ? '-' : ''}${state.sort.key}`
	});
	const { flights, total } = await get<FlightPage>(fetch, `/api/flights?${api}`);

	// Past the last page, e.g. after flights were removed: go to the last one.
	const pages = Math.max(1, Math.ceil(total / state.size));
	if (state.page > pages) redirect(307, `${url.pathname}${query({ ...state, page: pages })}`);

	return { flights, total, state };
}
