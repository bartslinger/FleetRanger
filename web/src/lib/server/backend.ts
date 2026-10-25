import { BACKEND_URL } from '$app/env/private';
import { error } from '@sveltejs/kit';
import type { Permissions } from '#lib/permissions.js';
import type { Area } from '#lib/ui/areas.js';
import type { DayStripFlight } from '#lib/ui/DayStrip.svelte';
import type { Track } from '#lib/ui/TrackMap.svelte';

export type { Area };

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
	group_id: string | null;
	group_name: string | null;
	/** Null: in no session. */
	session_id: string | null;
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

/** `GET /api/flights/devices`: the equipment, groups, pilots and areas of the
 * flights the user sees. */
export interface Devices {
	aircraft: Device[];
	controllers: Device[];
	/** No id: the flights without a group (the organisation queue); with
	 * `session=any`, the sessions' own groups, no id for the organisation. */
	groups: { id: string | null; name: string | null }[];
	/** No id: the flights without a pilot. */
	pilots: { id: string | null; name: string | null }[];
	areas: { id: string; name: string }[];
}

/** A flight's time and distance in one area. */
export interface FlightArea {
	id: string;
	name: string;
	air_time_s: number;
	distance_m: number;
}

/** What a drawing became, from `POST` or `PUT /api/areas`: one area, or more
 * with the same name where clipping split it (largest first), and the areas
 * it was clipped against. */
export interface SavedArea {
	areas: Area[];
	clipped_by: string[];
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
	group_id: string | null;
	group_name: string | null;
	/** Null: in no session. */
	session_id: string | null;
	track: Track;
	metrics: {
		max_altitude_m?: number;
		camera_serial?: string | null;
		battery_serial?: string | null;
	};
	/** The drone's and controller's pages; null where the user can't see them. */
	aircraft_id: string | null;
	rc_id: string | null;
	/** The battery's page; null for an archived flight, as batteries are seen
	 * through flights that aren't. */
	battery_id: string | null;
	battery_name: string | null;
	/** Set for an archived flight. */
	archived_at: string | null;
	/** The areas its track passes through, most time first; the rest is outside all areas. */
	areas: FlightArea[];
}

/** A flight as `GET /api/flights/{id}` returns it, for its page. */
export interface FlightPage extends FlightDetail {
	/** The first takeoff of its session's flights the user sees, to name the
	 * session as its page does; null in no session. */
	session_takeoff_at: string | null;
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
	/** The session's own group, which decides who changes it; null for the
	 * organisation. Its flights may be in other groups. */
	group_id: string | null;
	group_name: string | null;
	/** Each visible flight's times and takeoff point, in takeoff order: the day strip. */
	flight_times: DayStripFlight[];
}

/** One page of `GET /api/sessions`, newest first. */
export interface SessionPage {
	sessions: SessionSummary[];
	total: number;
}

/** A session with its visible flights in takeoff order, from `GET /api/sessions/{id}`. */
export interface Session {
	id: string;
	/** Its own group; null for the organisation. */
	group_id: string | null;
	group_name: string | null;
	flights: FlightDetail[];
	/** Oldest first. */
	comments: SessionComment[];
}

/** A comment on a session: a record of what happened. */
export interface SessionComment {
	id: string;
	author_id: string;
	author_name: string;
	body: string;
	created_at: string;
	/** When its author last changed it, if they did. */
	edited_at: string | null;
}

/** A session's neighbours in a list, newer and older, with their first
 * takeoff, from `GET /api/sessions/{id}/neighbours`. */
export interface SessionNeighbours {
	previous: string | null;
	previous_start: string | null;
	next: string | null;
	next_start: string | null;
}

/** An item's place in a list: the ones before and after it, its position
 * (from 1) and the list's length, e.g. `GET /api/flights/{id}/neighbours`. */
export interface Neighbours {
	previous: string | null;
	next: string | null;
	position: number;
	total: number;
}

/** The pilots and groups to choose from when changing flights, sessions and
 * equipment, from `GET /api/flights/choices`. Which changes the user may
 * make is in their permissions. */
export interface FlightChoices {
	/** Everyone who flies, by name, with the groups they fly for as a
	 * member; empty for someone who sets no pilots. */
	pilots: Pilot[];
	/** The groups where the user may move flights, sessions or equipment,
	 * or edit sessions, by name. */
	groups: { id: string; name: string }[];
}

/** Someone who can be a flight's pilot. */
export interface Pilot {
	id: string;
	name: string;
	/** The groups where they are a member with Pilot. */
	group_ids: string[];
}

/** One page of `GET /api/flights`. */
export interface FlightPage {
	flights: FlightSummary[];
	total: number;
}

/** A role in a group; Admin only on the whole organisation. */
export type GroupRole = 'viewer' | 'pilot' | 'editor';
export type OrganisationRole = GroupRole | 'admin';

/** A group, from `/api/groups`. */
export interface Group {
	id: string;
	name: string;
	/** Members who aren't archived. */
	members: number;
	/** What it holds; only a group without any of these can be deleted. */
	flights: number;
	sessions: number;
	drones: number;
	controllers: number;
}

/** A member who isn't archived, with their roles in the group. */
export interface Member {
	user_id: string;
	name: string;
	email: string | null;
	/** In the order viewer, pilot, editor. */
	roles: GroupRole[];
}

/** A group with its members, from `/api/groups/{id}`. */
export interface GroupDetail extends Group {
	member_list: Member[];
}

/** A user as admins manage them, from `/api/users`. */
export interface User {
	id: string;
	name: string;
	/** Roles on the whole organisation, in the order viewer, pilot, editor, admin. */
	roles: OrganisationRole[];
	/** The groups they are a member of, by name, with their roles there. */
	groups: { id: string; name: string; roles: GroupRole[] }[];
	/** The login's email; null if the user can't log in. */
	email: string | null;
	last_login_at: string | null;
	archived_at: string | null;
	/** Flights with this user as pilot. */
	flights: number;
}

/** A battery, from `/api/batteries`: seen through the flights the user may
 * see, with its figures and state over all its flights. */
export interface Battery {
	id: string;
	serial: string;
	name: string | null;
	flights: number;
	last_flight_at: string;
	/** The drone of its latest flight; `aircraft_id` null if not registered. */
	aircraft_id: string | null;
	aircraft_serial: string | null;
	aircraft_name: string | null;
	cycles: number | null;
	design_capacity_mah: number | null;
	/** What it held at its latest flight that reported it. */
	full_capacity_mah: number | null;
}

/** One flight of a battery, with what its log says about the battery. */
export interface BatteryFlight {
	flight_id: string;
	takeoff_at: string;
	air_time_s: number;
	aircraft_serial: string | null;
	aircraft_name: string | null;
	charge_start_pct: number | null;
	charge_end_pct: number | null;
	full_capacity_mah: number | null;
	design_capacity_mah: number | null;
	cycles: number | null;
	temperature_min_c: number | null;
	temperature_max_c: number | null;
	cell_voltage_min_v: number | null;
	cell_spread_max_v: number | null;
}

/** A battery with the flights the user sees, newest first, from `/api/batteries/{id}`. */
export interface BatteryDetail extends Battery {
	flight_list: BatteryFlight[];
	/** Its full charge capacity at each of all its flights, oldest first. */
	capacity_readings: { takeoff_at: string; full_capacity_mah: number }[];
}

/** A drone or remote controller, from `/api/aircraft` or `/api/remote-controllers`. */
export interface Equipment {
	id: string;
	serial: string;
	name: string | null;
	/** Null: directly in the organisation. */
	group_id: string | null;
	group_name: string | null;
	/** Over all its flights, also those the user doesn't see. */
	flights: number;
	air_time_s: number;
	last_flight_at: string | null;
	/** Who flew its last flight; null without flights or a pilot, or when the
	 * user doesn't see that flight. */
	last_pilot_name: string | null;
	/** Its last flight has a pilot, but the user doesn't see that flight. */
	last_pilot_hidden: boolean;
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

/** A code that can still pair a controller, from `GET /api/pairing-codes`. */
export interface PairingCode {
	id: string;
	code: string;
	created_by: string;
	/** The group the controller joins; null for the organisation. */
	group: string | null;
	created_at: string;
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

/** The logged-in user, from `GET /api/me`, or the user an admin impersonates. */
export interface Account {
	id: string;
	/** None for a user without a login, whom an admin impersonates. */
	email: string | null;
	name: string;
	/** What the user may do; see `#lib/permissions.js`. */
	permissions: Permissions;
	/** Whether apps (such as AI assistants) can connect to this install. */
	apps: boolean;
	/** The admin impersonating this user, who is the one logged in. */
	impersonated_by: { id: string; name: string } | null;
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
	/** Where the app sends the user back: who it really is. */
	host: string;
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
