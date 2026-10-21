import { TIME_ZONE } from '$app/env/private';
import { redirect } from '@sveltejs/kit';
import { get, type Session } from '#lib/server/backend.js';

/** A session's address: it opens over the cards of the month it was flown
 * in (`/sessions?month=…&session=…`), which is its page now. */
export async function load({ fetch, params }) {
	const session = await get<Session>(fetch, `/api/sessions/${params.id}`);
	const start = session.flights[0]?.takeoff_at;
	// en-CA writes dates as 2026-10-21.
	const day = start
		? new Intl.DateTimeFormat('en-CA', { timeZone: TIME_ZONE }).format(new Date(start))
		: null;
	const month = day ? `month=${day.slice(0, 7)}&` : '';
	redirect(307, `/sessions?${month}session=${encodeURIComponent(params.id)}`);
}
