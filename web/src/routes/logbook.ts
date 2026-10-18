// The logbook's state in the URL: page, page size and sort. Defaults are left
// out of the URL.

export const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;

export const SORTABLE = ['takeoff_at', 'air_time_s', 'distance_m', 'aircraft_serial', 'rc_serial'];
const DEFAULT_SORT = { key: 'takeoff_at', descending: true };

export interface LogbookState {
	page: number;
	size: number;
	sort: { key: string; descending: boolean };
}

export function parse(params: URLSearchParams): LogbookState {
	const page = Number.parseInt(params.get('page') ?? '', 10);
	const size = Number.parseInt(params.get('size') ?? '', 10);
	const sort = params.get('sort') ?? '';
	const key = sort.replace(/^-/, '');
	return {
		page: page >= 1 ? page : 1,
		size: PAGE_SIZES.includes(size) ? size : DEFAULT_SIZE,
		sort: SORTABLE.includes(key) ? { key, descending: sort.startsWith('-') } : DEFAULT_SORT
	};
}

/** The query string for a state, e.g. "?page=2&sort=air_time_s". */
export function query(state: LogbookState): string {
	const params = new URLSearchParams();
	if (state.page > 1) params.set('page', String(state.page));
	if (state.size !== DEFAULT_SIZE) params.set('size', String(state.size));
	const { key, descending } = state.sort;
	if (key !== DEFAULT_SORT.key || descending !== DEFAULT_SORT.descending) {
		params.set('sort', `${descending ? '-' : ''}${key}`);
	}
	const text = params.toString();
	return text ? `?${text}` : '';
}

export { DEFAULT_SORT };
