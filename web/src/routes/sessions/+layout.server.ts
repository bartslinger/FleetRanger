import { get, type Area, type Session } from '#lib/server/backend.js';

/**
 * The session open over the Sessions list (`?session=<id>`; user's
 * decision, 2026-10-21: one page, not a page of its own), and the areas
 * its map outlines. Here and not in the page's load, which reads only the
 * list's own parameters: opening, stepping through or closing a session
 * loads only this, not the list again.
 */
export async function load({ fetch, url }) {
	const id = url.searchParams.get('session');
	if (!id) return { openSession: null, areas: [] as Area[] };
	const [openSession, areas] = await Promise.all([
		get<Session>(fetch, `/api/sessions/${id}`),
		get<Area[]>(fetch, '/api/areas')
	]);
	return { openSession, areas };
}
