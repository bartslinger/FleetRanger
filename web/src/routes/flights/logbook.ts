// The logbook's state in the URL: page, page size, sort and filters. Defaults
// are left out of the URL.

import { isDay, nextDay, startOfDay } from '#lib/zone.js';

export const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;

export const SORTABLE = ['takeoff_at', 'air_time_s', 'distance_m', 'aircraft_serial', 'rc_serial'];
const DEFAULT_SORT = { key: 'takeoff_at', descending: true };

export interface LogbookState {
	page: number;
	size: number;
	sort: { key: string; descending: boolean };
	/** Only the flights of these drone serials; all if empty. */
	drones: string[];
	/** Only the flights of these controller serials; all if empty. */
	controllers: string[];
	/** Only the flights of these team ids, "none" for those without a team; all if empty. */
	teams: string[];
	/** Only the flights in no session, the ones still to be grouped. */
	withoutSession: boolean;
	/** With `withoutSession`: those flights as suggested sessions, not a list. */
	suggestions: boolean;
	/** Only flights taking off on this day ("2026-10-12"), or from it to `to`. */
	from: string | null;
	/** The last day, with `from`. */
	to: string | null;
}

/** Serials, comma-separated in the URL. */
const list = (value: string | null) => (value ?? '').split(',').filter(Boolean);

export function parse(params: URLSearchParams): LogbookState {
	const page = Number.parseInt(params.get('page') ?? '', 10);
	const size = Number.parseInt(params.get('size') ?? '', 10);
	const sort = params.get('sort') ?? '';
	const key = sort.replace(/^-/, '');
	const from = params.get('from');
	const to = params.get('to');
	return {
		page: page >= 1 ? page : 1,
		size: PAGE_SIZES.includes(size) ? size : DEFAULT_SIZE,
		sort: SORTABLE.includes(key) ? { key, descending: sort.startsWith('-') } : DEFAULT_SORT,
		drones: list(params.get('drone')),
		controllers: list(params.get('controller')),
		teams: list(params.get('team')),
		withoutSession: params.get('session') === 'none',
		suggestions: params.get('session') === 'none' && params.get('view') === 'suggestions',
		from: isDay(from) ? from : null,
		to: isDay(from) && isDay(to) && to > from ? to : null
	};
}

/** The query string for a state, e.g. "?page=2&sort=air_time_s". */
export function query(state: LogbookState): string {
	const params = new URLSearchParams();
	if (state.from) params.set('from', state.from);
	if (state.from && state.to) params.set('to', state.to);
	if (state.drones.length > 0) params.set('drone', state.drones.join(','));
	if (state.controllers.length > 0) params.set('controller', state.controllers.join(','));
	if (state.teams.length > 0) params.set('team', state.teams.join(','));
	if (state.withoutSession) params.set('session', 'none');
	if (state.withoutSession && state.suggestions) params.set('view', 'suggestions');
	if (state.page > 1) params.set('page', String(state.page));
	if (state.size !== DEFAULT_SIZE) params.set('size', String(state.size));
	const { key, descending } = state.sort;
	if (key !== DEFAULT_SORT.key || descending !== DEFAULT_SORT.descending) {
		params.set('sort', `${descending ? '-' : ''}${key}`);
	}
	const text = params.toString();
	return text ? `?${text}` : '';
}

/** The backend's query for the list: its page, filters and order. Days are
 * turned into instants in the organisation's time zone. */
export function apiQuery(state: LogbookState, timeZone: string): URLSearchParams {
	const api = new URLSearchParams({
		page: String(state.page),
		size: String(state.size),
		sort: `${state.sort.descending ? '-' : ''}${state.sort.key}`
	});
	if (state.drones.length > 0) api.set('aircraft', state.drones.join(','));
	if (state.controllers.length > 0) api.set('controller', state.controllers.join(','));
	if (state.teams.length > 0) api.set('team', state.teams.join(','));
	if (state.withoutSession) api.set('session', 'none');
	if (state.from) {
		api.set('from', startOfDay(state.from, timeZone).toISOString());
		api.set('to', startOfDay(nextDay(state.to ?? state.from), timeZone).toISOString());
	}
	return api;
}

/** The logbook with only these filters, e.g. a drone's flights. */
export function filtered(filters: { drone?: string; controller?: string }): string {
	return query({
		page: 1,
		size: DEFAULT_SIZE,
		sort: DEFAULT_SORT,
		drones: filters.drone ? [filters.drone] : [],
		controllers: filters.controller ? [filters.controller] : [],
		teams: [],
		withoutSession: false,
		suggestions: false,
		from: null,
		to: null
	});
}

export { DEFAULT_SORT };
