// The logbook's state in the URL: page, page size, sort and filters. Defaults
// are left out of the URL.

export const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;

export const SORTABLE = ['takeoff_at', 'air_time_s', 'distance_m', 'aircraft_serial', 'rc_serial'];
const DEFAULT_SORT = { key: 'takeoff_at', descending: true };

export interface LogbookState {
	page: number;
	size: number;
	sort: { key: string; descending: boolean };
	/** Only the flights of this drone serial. */
	drone: string | null;
	/** Only the flights of this controller serial. */
	controller: string | null;
	/** Only the flights in no session, the ones still to be grouped. */
	withoutSession: boolean;
}

export function parse(params: URLSearchParams): LogbookState {
	const page = Number.parseInt(params.get('page') ?? '', 10);
	const size = Number.parseInt(params.get('size') ?? '', 10);
	const sort = params.get('sort') ?? '';
	const key = sort.replace(/^-/, '');
	return {
		page: page >= 1 ? page : 1,
		size: PAGE_SIZES.includes(size) ? size : DEFAULT_SIZE,
		sort: SORTABLE.includes(key) ? { key, descending: sort.startsWith('-') } : DEFAULT_SORT,
		drone: params.get('drone') || null,
		controller: params.get('controller') || null,
		withoutSession: params.get('session') === 'none'
	};
}

/** The query string for a state, e.g. "?page=2&sort=air_time_s". */
export function query(state: LogbookState): string {
	const params = new URLSearchParams();
	if (state.drone) params.set('drone', state.drone);
	if (state.controller) params.set('controller', state.controller);
	if (state.withoutSession) params.set('session', 'none');
	if (state.page > 1) params.set('page', String(state.page));
	if (state.size !== DEFAULT_SIZE) params.set('size', String(state.size));
	const { key, descending } = state.sort;
	if (key !== DEFAULT_SORT.key || descending !== DEFAULT_SORT.descending) {
		params.set('sort', `${descending ? '-' : ''}${key}`);
	}
	const text = params.toString();
	return text ? `?${text}` : '';
}

/** The logbook with only these filters, e.g. a drone's flights. */
export function filtered(filters: { drone?: string; controller?: string }): string {
	return query({
		page: 1,
		size: DEFAULT_SIZE,
		sort: DEFAULT_SORT,
		drone: filters.drone ?? null,
		controller: filters.controller ?? null,
		withoutSession: false
	});
}

export { DEFAULT_SORT };
