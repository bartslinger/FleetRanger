import { BACKEND_URL } from '$app/env/private';
import { error } from '@sveltejs/kit';
import type { Track } from '#lib/ui/TrackMap.svelte';

/** A flight as listed by `GET /api/flights`. */
export interface FlightSummary {
	id: string;
	takeoff_at: string;
	landed_at: string;
	air_time_s: number;
	distance_m: number;
	aircraft_serial: string | null;
	aircraft_name: string | null;
	rc_serial: string | null;
	rc_name: string | null;
	pilot_id: string | null;
	pilot_name: string | null;
	/** Null: in the organisation's queue. */
	team_id: string | null;
	team_name: string | null;
	/** Set for an archived flight: left out everywhere but the archived filter. */
	archived_at: string | null;
}

/** Flights without a session that look like one, from `GET /api/flights/suggestions`. */
export interface Suggestion {
	/** The first flight's id. */
	key: string;
	/** In takeoff order. */
	flights: FlightSummary[];
}

/** One page of suggestions, newest first. */
export interface SuggestionPage {
	suggestions: Suggestion[];
	total: number;
}

/** One suggestion with its tracks, from `GET /api/flights/suggestions/{key}`. */
export interface SuggestionDetail {
	key: string;
	/** In takeoff order. */
	flights: FlightDetail[];
	/** The newer and older suggestions' keys. */
	previous: string | null;
	next: string | null;
	/** From 1, newest first. */
	position: number;
	total: number;
}

/** A drone or controller to filter the logbook by. */
export interface Device {
	serial: string;
	name: string | null;
}

/** `GET /api/flights/devices`: the equipment of the flights the user sees. */
export interface Devices {
	aircraft: Device[];
	controllers: Device[];
	/** No id: the flights without a team (the organisation queue). */
	teams: { id: string | null; name: string | null }[];
	/** No id: the flights without a pilot. */
	pilots: { id: string | null; name: string | null }[];
}

/** One flight with its track, from `GET /api/flights/{id}`. */
export interface FlightDetail {
	id: string;
	takeoff_at: string;
	landed_at: string;
	air_time_s: number;
	distance_m: number;
	aircraft_serial: string | null;
	aircraft_name: string | null;
	rc_serial: string | null;
	rc_name: string | null;
	pilot_id: string | null;
	pilot_name: string | null;
	/** Null: in the organisation's queue. */
	team_id: string | null;
	team_name: string | null;
	track: Track;
	metrics: {
		max_altitude_m?: number;
		camera_serial?: string | null;
		battery_serial?: string | null;
	};
	/** The drone's and controller's pages; null where the user can't see them. */
	aircraft_id: string | null;
	rc_id: string | null;
	/** Set for an archived flight. */
	archived_at: string | null;
}

/** A session as listed by `GET /api/sessions`: what its visible flights add up to. */
export interface SessionSummary {
	id: string;
	first_takeoff_at: string;
	last_landed_at: string;
	flights: number;
	air_time_s: number;
	distance_m: number;
	/** Names in alphabetical order. */
	pilots: string[];
	teams: string[];
	team_ids: string[];
}

/** One page of `GET /api/sessions`, newest first. */
export interface SessionPage {
	sessions: SessionSummary[];
	total: number;
}

/** A session with its visible flights in takeoff order, from `GET /api/sessions/{id}`. */
export interface Session {
	id: string;
	flights: FlightDetail[];
}

/** An item's place in a list: the ones before and after it, its position
 * (from 1) and the list's length, e.g. `GET /api/sessions/{id}/neighbours`. */
export interface Neighbours {
	previous: string | null;
	next: string | null;
	position: number;
	total: number;
}

/** What the logged-in account can change in the logbook, from `GET /api/flights/choices`. */
export interface FlightChoices {
	/** The teams whose flights a member can change. */
	own_team_ids: string[];
	/** Admins change any flight and move flights between teams. */
	admin: boolean;
	/** A pilot in several teams is a choice per team: the flights move to it. */
	pilots: { id: string; name: string; team_id: string; team_name: string }[];
	teams: { id: string; name: string }[];
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
	/** Users in the team that aren't archived. */
	members: number;
}

/** A user as admins manage them, from `/api/users`. */
export interface User {
	id: string;
	name: string;
	role: 'member' | 'all_flights' | 'admin';
	/** The teams the user flies for, by name. */
	team_ids: string[];
	team_names: string[];
	/** The login's email; null if the user can't log in. */
	email: string | null;
	last_login_at: string | null;
	archived_at: string | null;
	/** Flights with this user as pilot. */
	flights: number;
}

/** A drone or remote controller, from `/api/aircraft` or `/api/remote-controllers`. */
export interface Equipment {
	id: string;
	serial: string;
	name: string | null;
	team_id: string | null;
	team_name: string | null;
	holder_user_id: string | null;
	holder_name: string | null;
	last_flight_at: string | null;
	/** Whether a controller is paired with the install; null for drones. */
	paired: boolean | null;
	/** When a controller last synced, also without logs; null for drones and never. */
	synced_at: string | null;
	/** The app version a controller reported when it last synced. */
	app_version: string | null;
}

/** The controller app's download, from `GET /api/download` (public). */
export interface Download {
	/** Null when the install has no APK. */
	android: { size_bytes: number; version_code: number | null; version_name: string | null } | null;
}

/** A controller waiting to be paired, from `GET /api/pairing`: without its code. */
export interface WaitingController {
	serial: string;
	model: string;
	expires_at: string;
}

/** Progress through the current batch of logs, from `GET /api/logs/processing` (admins). */
export interface ProcessingStatus {
	/** Zero, with everything else empty, when nothing is waiting. */
	pending: number;
	/** Processed so far in this batch, failed ones included. */
	processed: number;
	failed: number;
	current: {
		/** Relative to the flight log directory. */
		path: string;
		stage: 'reading' | 'requesting_keys' | 'decoding' | 'saving';
		started_at: string;
	} | null;
	paused: { reason: string; until: string } | null;
	/** The latest processed logs, newest first. */
	latest: {
		path: string;
		status: 'done' | 'failed';
		error: string | null;
		/** Null if the aircraft never took off. */
		flight_id: string | null;
	}[];
}

/** The logged-in user, from `GET /api/me`. */
export interface Account {
	id: string;
	email: string;
	name: string;
	role: 'member' | 'all_flights' | 'admin';
	/** Whether apps (such as AI assistants) can connect to this install. */
	apps: boolean;
}

/** An app asking to connect, from `POST /api/oauth/request`. */
export interface AppRequest {
	/** As the app named itself. */
	app: string;
	/** Where the app sends the user back: who it really is. */
	host: string;
	scopes: string[];
}

/** An app the user connected, from `GET /api/oauth/connections`. */
export interface Connection {
	id: string;
	app: string;
	scopes: string[];
	created_at: string;
	last_used_at: string;
}

/** The user of the browser's login session, or null if it isn't logged in. */
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

/** Deletes something; like `send`, a refusal (400 or 409) comes back with its message. */
export async function remove(
	fetch: typeof globalThis.fetch,
	path: string
): Promise<SendResult<null>> {
	let response: Response;
	try {
		response = await fetch(`${BACKEND_URL}${path}`, { method: 'DELETE' });
	} catch {
		error(503, 'The backend is not reachable.');
	}
	if (response.status === 400 || response.status === 409) {
		return { ok: false, status: response.status, message: await response.text() };
	}
	if (response.status === 403 || response.status === 404) error(response.status);
	if (!response.ok) error(502, `The backend answered ${response.status}.`);
	return { ok: true, data: null };
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
