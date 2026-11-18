// The Flights page's state in the URL: view, week or month, page, page size,
// sort and filters. Defaults are left out of the URL.

import { days, periodOf, today, type Period } from '#lib/period.js';
import { isDay, nextDay, startOfDay } from '#lib/zone.js';

export const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;
/** The backend's largest page: the cards ask for their whole period in pages of it. */
export const PERIOD_SIZE = 100;

export const SORTABLE = ['takeoff_at', 'air_time_s', 'distance_m', 'aircraft_serial', 'rc_serial'];
const DEFAULT_SORT = { key: 'takeoff_at', descending: true };

export interface LogbookState {
	/** Cards of a week or month beside a map (the default), or the table. */
	view: 'cards' | 'table';
	/** The cards' week or month; the table uses `from` and `to`. */
	period: Period;
	/** The month a plain address opens, left out of the URL: the latest with
	 * activity, which the load sets (user's decision, 2026-11-18), else this month. */
	defaultMonth: string;
	page: number;
	size: number;
	sort: { key: string; descending: boolean };
	/** Only the flights of these pilots' ids, "none" for those without a pilot; all if empty. */
	pilots: string[];
	/** Only the flights of these drone serials; all if empty. */
	drones: string[];
	/** Only the flights of these controller serials; all if empty. */
	controllers: string[];
	/** Only the flights of these group ids, "none" for those without a group; all if empty. */
	groups: string[];
	/** Only the flights with time in any of these areas' ids; all if empty. */
	areas: string[];
	/** Only the flights in no session, the ones still to be grouped. */
	withoutSession: boolean;
	/** With `withoutSession`: those flights as suggested sessions, instead of cards or a table. */
	suggestions: boolean;
	/** Only archived flights, which are otherwise left out; never in a session. */
	archived: boolean;
	/** Only flights taking off on this day ("2026-10-12"), or from it to `to`. */
	from: string | null;
	/** The last day, with `from`. */
	to: string | null;
}

/** Serials, comma-separated in the URL. */
const list = (value: string | null) => (value ?? '').split(',').filter(Boolean);

/** The cards' period: `week=2026-10-12` (a Monday) or `month=2026-10`, else this month. */
function period(params: URLSearchParams, defaultMonth: string): Period {
	const week = params.get('week');
	if (isDay(week)) return periodOf('week', week);
	const month = params.get('month');
	if (month && isDay(`${month}-01`)) return periodOf('month', `${month}-01`);
	return periodOf('month', defaultMonth);
}

/** With `areas` false (the organisation doesn't use them), an area filter in
 * the URL is left out: it would filter by something not shown. */
export function parse(params: URLSearchParams, timeZone: string, areas = true): LogbookState {
	const page = Number.parseInt(params.get('page') ?? '', 10);
	const size = Number.parseInt(params.get('size') ?? '', 10);
	const sort = params.get('sort') ?? '';
	const key = sort.replace(/^-/, '');
	const from = params.get('from');
	const to = params.get('to');
	const archived = params.get('status') === 'archived';
	const defaultMonth = periodOf('month', today(timeZone)).start;
	return {
		view: params.get('view') === 'table' ? 'table' : 'cards',
		period: period(params, defaultMonth),
		defaultMonth,
		page: page >= 1 ? page : 1,
		size: PAGE_SIZES.includes(size) ? size : DEFAULT_SIZE,
		sort: SORTABLE.includes(key) ? { key, descending: sort.startsWith('-') } : DEFAULT_SORT,
		pilots: list(params.get('pilot')),
		drones: list(params.get('drone')),
		controllers: list(params.get('controller')),
		groups: list(params.get('group')),
		areas: areas ? list(params.get('area')) : [],
		withoutSession: !archived && params.get('session') === 'none',
		suggestions:
			!archived && params.get('session') === 'none' && params.get('view') === 'suggestions',
		archived,
		from: isDay(from) ? from : null,
		to: isDay(from) && isDay(to) && to > from ? to : null
	};
}

/** Whether the state shows the cards: not suggestions, not the table. */
export const showsCards = (state: LogbookState) => !state.suggestions && state.view === 'cards';

/** The query string for a state, e.g. "?view=table&page=2&sort=air_time_s";
 * with a flight, that one open over the cards. */
export function query(state: LogbookState, flight?: string): string {
	const params = new URLSearchParams();
	const cards = showsCards(state);
	// The cards' period stands in for the days.
	if (!cards && state.from) params.set('from', state.from);
	if (!cards && state.from && state.to) params.set('to', state.to);
	if (state.pilots.length > 0) params.set('pilot', state.pilots.join(','));
	if (state.drones.length > 0) params.set('drone', state.drones.join(','));
	if (state.controllers.length > 0) params.set('controller', state.controllers.join(','));
	if (state.groups.length > 0) params.set('group', state.groups.join(','));
	if (state.areas.length > 0) params.set('area', state.areas.join(','));
	if (state.withoutSession) params.set('session', 'none');
	if (state.archived) params.set('status', 'archived');
	if (state.withoutSession && state.suggestions) params.set('view', 'suggestions');
	else if (state.view === 'table') params.set('view', 'table');
	else if (state.period.unit === 'week') params.set('week', state.period.start);
	else if (state.period.start !== state.defaultMonth) {
		params.set('month', state.period.start.slice(0, 7));
	}
	if (!cards) {
		if (state.page > 1) params.set('page', String(state.page));
		if (state.size !== DEFAULT_SIZE) params.set('size', String(state.size));
		const { key, descending } = state.sort;
		if (key !== DEFAULT_SORT.key || descending !== DEFAULT_SORT.descending) {
			params.set('sort', `${descending ? '-' : ''}${key}`);
		}
	}
	if (flight) params.set('flight', flight);
	const text = params.toString();
	return text ? `?${text}` : '';
}

/** The backend's filters, without page, size and order. Days are turned
 * into instants in the organisation's time zone. */
export function apiFilters(state: LogbookState, timeZone: string): URLSearchParams {
	const api = new URLSearchParams();
	if (state.pilots.length > 0) api.set('pilot', state.pilots.join(','));
	if (state.drones.length > 0) api.set('aircraft', state.drones.join(','));
	if (state.controllers.length > 0) api.set('controller', state.controllers.join(','));
	if (state.groups.length > 0) api.set('group', state.groups.join(','));
	if (state.areas.length > 0) api.set('area', state.areas.join(','));
	if (state.withoutSession) api.set('session', 'none');
	// A flight's neighbours are then those of its suggestion.
	if (state.withoutSession && state.suggestions) api.set('view', 'suggestions');
	if (state.archived) api.set('archived', 'true');
	if (state.from) {
		api.set('from', startOfDay(state.from, timeZone).toISOString());
		api.set('to', startOfDay(nextDay(state.to ?? state.from), timeZone).toISOString());
	}
	return api;
}

/** The backend's query for the list: its page, filters and order. The cards
 * show their whole period, newest first, with the flights' tracks. */
export function apiQuery(state: LogbookState, timeZone: string): URLSearchParams {
	if (showsCards(state)) {
		const api = apiFilters({ ...state, ...days(state.period) }, timeZone);
		api.set('page', '1');
		api.set('size', String(PERIOD_SIZE));
		api.set('sort', '-takeoff_at');
		return api;
	}
	const api = apiFilters(state, timeZone);
	api.set('page', String(state.page));
	api.set('size', String(state.size));
	api.set('sort', `${state.sort.descending ? '-' : ''}${state.sort.key}`);
	return api;
}

/** The Flights page with only these filters, e.g. a drone's flights: its table. */
export function filtered(filters: { drone?: string; controller?: string }): string {
	return query({
		view: 'table',
		period: periodOf('month', '2000-01-01'),
		defaultMonth: '2000-01-01',
		page: 1,
		size: DEFAULT_SIZE,
		sort: DEFAULT_SORT,
		pilots: [],
		drones: filters.drone ? [filters.drone] : [],
		controllers: filters.controller ? [filters.controller] : [],
		groups: [],
		areas: [],
		withoutSession: false,
		suggestions: false,
		archived: false,
		from: null,
		to: null
	});
}

export { DEFAULT_SORT };
