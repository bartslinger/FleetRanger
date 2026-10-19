// The Sessions list's state in the URL: page, page size, view and the
// Flights list's filters. Defaults are left out of the URL.

import type { Filters } from '#lib/ListFilters.svelte';
import { isDay, nextDay, startOfDay } from '#lib/zone.js';

export const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;

export interface SessionListState extends Filters {
	page: number;
	size: number;
	view: 'cards' | 'table';
}

/** Serials and ids, comma-separated in the URL. */
const list = (value: string | null) => (value ?? '').split(',').filter(Boolean);

export function parse(params: URLSearchParams): SessionListState {
	const page = Number.parseInt(params.get('page') ?? '', 10);
	const size = Number.parseInt(params.get('size') ?? '', 10);
	const from = params.get('from');
	const to = params.get('to');
	return {
		page: page >= 1 ? page : 1,
		size: PAGE_SIZES.includes(size) ? size : DEFAULT_SIZE,
		view: params.get('view') === 'table' ? 'table' : 'cards',
		drones: list(params.get('drone')),
		controllers: list(params.get('controller')),
		teams: list(params.get('team')),
		from: isDay(from) ? from : null,
		to: isDay(from) && isDay(to) && to > from ? to : null
	};
}

/** The query string for a state, e.g. "?drone=A1&view=table". */
export function query(state: SessionListState): string {
	const params = new URLSearchParams();
	if (state.from) params.set('from', state.from);
	if (state.from && state.to) params.set('to', state.to);
	if (state.drones.length > 0) params.set('drone', state.drones.join(','));
	if (state.controllers.length > 0) params.set('controller', state.controllers.join(','));
	if (state.teams.length > 0) params.set('team', state.teams.join(','));
	if (state.page > 1) params.set('page', String(state.page));
	if (state.size !== DEFAULT_SIZE) params.set('size', String(state.size));
	if (state.view !== 'cards') params.set('view', state.view);
	const text = params.toString();
	return text ? `?${text}` : '';
}

/** The backend's filters, as the Flights list's: days become instants in the
 * organisation's time zone. Without page and size, for a session's place in
 * the list and the filters' options. */
export function apiFilters(state: Filters, timeZone: string): URLSearchParams {
	const api = new URLSearchParams();
	if (state.drones.length > 0) api.set('aircraft', state.drones.join(','));
	if (state.controllers.length > 0) api.set('controller', state.controllers.join(','));
	if (state.teams.length > 0) api.set('team', state.teams.join(','));
	if (state.from) {
		api.set('from', startOfDay(state.from, timeZone).toISOString());
		api.set('to', startOfDay(nextDay(state.to ?? state.from), timeZone).toISOString());
	}
	return api;
}
