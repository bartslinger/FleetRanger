import { TIME_ZONE } from '$app/env/private';
import { isHttpError } from '@sveltejs/kit';
import { get, type Area, type Session, type SessionNeighbours } from '#lib/server/backend.js';
import { apiFilters, parse } from './list.js';

/**
 * The session open over the Sessions list (`?session=<id>`; user's
 * decision, 2026-10-21: one page, not a page of its own), the areas its map
 * outlines, and the sessions before and after it. Here and not in the
 * page's load, which reads only the list's own parameters: opening,
 * stepping through or closing a session loads only this, not the list again.
 */
export async function load({ fetch, url }) {
	const id = url.searchParams.get('session');
	if (!id) return { openSession: null, areas: [] as Area[], neighbours: null };
	// Stepping goes on into the weeks or months around (user's decision,
	// 2026-10-21) with the list's other filters, so without its days.
	const state = parse(url.searchParams, TIME_ZONE);
	const filters = apiFilters({ ...state, from: null, to: null }, TIME_ZONE);
	const [openSession, areas, neighbours] = await Promise.all([
		get<Session>(fetch, `/api/sessions/${id}`),
		get<Area[]>(fetch, '/api/areas'),
		// None if the session no longer passes the filters, e.g. its pilot changed.
		get<SessionNeighbours>(fetch, `/api/sessions/${id}/neighbours?${filters}`).catch((failure) => {
			if (isHttpError(failure, 404)) return null;
			throw failure;
		})
	]);
	return { openSession, areas, neighbours };
}
