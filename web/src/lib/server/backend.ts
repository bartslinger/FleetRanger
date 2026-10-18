import { BACKEND_URL } from '$app/env/private';
import { error } from '@sveltejs/kit';

/** A flight as listed by `GET /api/flights`. */
export interface FlightSummary {
	id: string;
	takeoff_at: string;
	landed_at: string;
	air_time_s: number;
	distance_m: number;
	aircraft_serial: string | null;
	rc_serial: string | null;
}

/** One page of `GET /api/flights`. */
export interface FlightPage {
	flights: FlightSummary[];
	total: number;
}

/** A team, from `/api/teams`. */
export interface Team {
	id: string;
	name: string;
	archived_at: string | null;
	/** Pilots in the team that aren't archived. */
	pilots: number;
}

/** A pilot, from `/api/pilots`. */
export interface Pilot {
	id: string;
	name: string;
	team_id: string | null;
	team_name: string | null;
	/** The linked user. */
	account_id: string | null;
	account_email: string | null;
	archived_at: string | null;
}

/** An account as admins manage it, from `/api/accounts`; "user" in the UI. */
export interface User {
	id: string;
	email: string;
	name: string | null;
	role: 'member' | 'all_flights' | 'admin';
	disabled_at: string | null;
	pilot_id: string | null;
	pilot_name: string | null;
	last_login_at: string | null;
}

/** The logged-in account, from `GET /api/me`. */
export interface Account {
	email: string;
	name: string | null;
	role: 'member' | 'all_flights' | 'admin';
}

/** The account of the browser's login session, or null if it isn't logged in. */
export async function me(fetch: typeof globalThis.fetch): Promise<Account | null> {
	let response: Response;
	try {
		response = await fetch(`${BACKEND_URL}/api/me`);
	} catch {
		error(503, 'The backend is not reachable.');
	}
	if (response.status === 401) return null;
	if (!response.ok) error(502, `The backend answered ${response.status}.`);
	return response.json();
}

/** The result of a change sent to the backend. */
export type SendResult<T> =
	{ ok: true; data: T } | { ok: false; status: 400 | 409; message: string };

/**
 * Sends a change (POST or PUT) with a JSON body. A rejected change (400 or 409)
 * comes back with the backend's message, written for the user, so a form can
 * show it; any other failure fails the request.
 */
export async function send<T>(
	fetch: typeof globalThis.fetch,
	method: 'POST' | 'PUT',
	path: string,
	body?: unknown
): Promise<SendResult<T>> {
	let response: Response;
	try {
		response = await fetch(`${BACKEND_URL}${path}`, {
			method,
			headers: body === undefined ? {} : { 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
	} catch {
		error(503, 'The backend is not reachable.');
	}
	if (response.status === 400 || response.status === 409) {
		return { ok: false, status: response.status, message: await response.text() };
	}
	if (response.status === 403 || response.status === 404) error(response.status);
	if (!response.ok) error(502, `The backend answered ${response.status}.`);
	return { ok: true, data: await response.json() };
}

/** Fetches JSON from the backend, failing the request if the backend does. */
export async function get<T>(fetch: typeof globalThis.fetch, path: string): Promise<T> {
	let response: Response;
	try {
		response = await fetch(`${BACKEND_URL}${path}`);
	} catch {
		error(503, 'The backend is not reachable.');
	}
	if (response.status === 403 || response.status === 404) error(response.status);
	if (!response.ok) error(502, `The backend answered ${response.status}.`);
	return response.json();
}
