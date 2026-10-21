// The Sessions list's state in the URL: page, page size, view and the
// Flights list's filters. Defaults are left out of the URL.

import type { Filters } from '#lib/ListFilters.svelte';
import { days, periodOf, today, type Period } from '#lib/period.js';
import { isDay, nextDay, startOfDay } from '#lib/zone.js';

export const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;
/** The cards show a whole period on one page, up to the backend's largest page. */
export const PERIOD_SIZE = 100;

export interface SessionListState extends Filters {
	page: number;
	size: number;
	view: 'cards' | 'table';
	/** The cards' week or month; the table uses `from` and `to`. */
	period: Period;
	/** The current month, the default, left out of the URL. */
	thisMonth: string;
}

/** Serials and ids, comma-separated in the URL. */
const list = (value: string | null) => (value ?? '').split(',').filter(Boolean);

/** The cards' period: `week=2026-10-12` (a Monday) or `month=2026-10`, else this month. */
function period(params: URLSearchParams, thisMonth: string): Period {
	const week = params.get('week');
	if (isDay(week)) return periodOf('week', week);
	const month = params.get('month');
	if (month && isDay(`${month}-01`)) return periodOf('month', `${month}-01`);
	return periodOf('month', thisMonth);
}

export function parse(params: URLSearchParams, timeZone: string): SessionListState {
	const page = Number.parseInt(params.get('page') ?? '', 10);
	const size = Number.parseInt(params.get('size') ?? '', 10);
	const from = params.get('from');
	const to = params.get('to');
	const thisMonth = periodOf('month', today(timeZone)).start;
	return {
		period: period(params, thisMonth),
		thisMonth,
		page: page >= 1 ? page : 1,
		size: PAGE_SIZES.includes(size) ? size : DEFAULT_SIZE,
		view: params.get('view') === 'table' ? 'table' : 'cards',
		pilots: list(params.get('pilot')),
		drones: list(params.get('drone')),
		controllers: list(params.get('controller')),
		teams: list(params.get('team')),
		areas: list(params.get('area')),
		from: isDay(from) ? from : null,
		to: isDay(from) && isDay(to) && to > from ? to : null
	};
}

/** The query string for a state, e.g. "?drone=A1&view=table"; with a
 * session, that one open over the list. (`parse` leaves `session` out: the
 * list's load doesn't depend on it, the layout's loads the session.) */
export function query(state: SessionListState, session?: string): string {
	const params = new URLSearchParams();
	if (state.from) params.set('from', state.from);
	if (state.from && state.to) params.set('to', state.to);
	if (state.pilots.length > 0) params.set('pilot', state.pilots.join(','));
	if (state.drones.length > 0) params.set('drone', state.drones.join(','));
	if (state.controllers.length > 0) params.set('controller', state.controllers.join(','));
	if (state.teams.length > 0) params.set('team', state.teams.join(','));
	if (state.areas.length > 0) params.set('area', state.areas.join(','));
	if (state.page > 1) params.set('page', String(state.page));
	if (state.size !== DEFAULT_SIZE) params.set('size', String(state.size));
	if (state.view !== 'cards') params.set('view', state.view);
	else if (state.period.unit === 'week') params.set('week', state.period.start);
	else if (state.period.start !== state.thisMonth) {
		params.set('month', state.period.start.slice(0, 7));
	}
	if (session) params.set('session', session);
	const text = params.toString();
	return text ? `?${text}` : '';
}

/** The backend's filters, as the Flights list's: days become instants in the
 * organisation's time zone. Without page and size, for a session's place in
 * the list and the filters' options. */
export function apiFilters(state: Filters, timeZone: string): URLSearchParams {
	const api = new URLSearchParams();
	if (state.pilots.length > 0) api.set('pilot', state.pilots.join(','));
	if (state.drones.length > 0) api.set('aircraft', state.drones.join(','));
	if (state.controllers.length > 0) api.set('controller', state.controllers.join(','));
	if (state.teams.length > 0) api.set('team', state.teams.join(','));
	if (state.areas.length > 0) api.set('area', state.areas.join(','));
	if (state.from) {
		api.set('from', startOfDay(state.from, timeZone).toISOString());
		api.set('to', startOfDay(nextDay(state.to ?? state.from), timeZone).toISOString());
	}
	return api;
}

/** The list's filters for the backend: the cards' period, or the table's days. */
export function listFilters(state: SessionListState, timeZone: string): URLSearchParams {
	return apiFilters(state.view === 'cards' ? { ...state, ...days(state.period) } : state, timeZone);
}
