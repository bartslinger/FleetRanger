import { get } from './backend.js';

/** Pages asked for at once: a long period doesn't take all the backend's connections. */
const AT_ONCE = 3;

/**
 * Every page of a paged list: the first, which gives the total, then the
 * rest a few at a time. For a view that promises all of something, such as
 * the cards' whole week or month, where the backend's pages hold at most
 * 100. `params` asks for page 1 and the page size.
 */
export async function allPages<P extends { total: number }>(
	fetch: typeof globalThis.fetch,
	path: string,
	params: URLSearchParams
): Promise<P[]> {
	const first = await get<P>(fetch, `${path}?${params}`);
	const size = Number(params.get('size'));
	const count = size > 0 ? Math.ceil(first.total / size) : 1;
	const pages = [first];
	for (let next = 2; next <= count; next += AT_ONCE) {
		const batch = Array.from({ length: Math.min(AT_ONCE, count - next + 1) }, (_, index) => {
			const page = new URLSearchParams(params);
			page.set('page', String(next + index));
			return get<P>(fetch, `${path}?${page}`);
		});
		pages.push(...(await Promise.all(batch)));
	}
	return pages;
}

/** Items by id, each once: an item added between two pages' requests moves
 * the last of a page onto the next. */
export function once<T extends { id: string }>(items: T[]): T[] {
	const seen = new Set<string>();
	return items.filter((item) => !seen.has(item.id) && Boolean(seen.add(item.id)));
}
