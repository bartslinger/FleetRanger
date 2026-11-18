import { get } from './backend.js';

/**
 * Every page of a paged list: the first, which gives the total, then the
 * rest together. For a view that promises all of something, such as the
 * cards' whole week or month, where the backend's pages hold at most 100.
 * `params` asks for page 1 and the page size.
 */
export async function allPages<P extends { total: number }>(
	fetch: typeof globalThis.fetch,
	path: string,
	params: URLSearchParams
): Promise<P[]> {
	const first = await get<P>(fetch, `${path}?${params}`);
	const size = Number(params.get('size'));
	const count = size > 0 ? Math.ceil(first.total / size) : 1;
	const rest = await Promise.all(
		Array.from({ length: Math.max(0, count - 1) }, (_, index) => {
			const page = new URLSearchParams(params);
			page.set('page', String(index + 2));
			return get<P>(fetch, `${path}?${page}`);
		})
	);
	return [first, ...rest];
}
