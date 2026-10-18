// Account roles as the web app shows them.

import type { User } from '#lib/server/backend.js';

export type Role = User['role'];

export const ROLE_LABELS: Record<Role, string> = {
	member: 'Member',
	all_flights: 'All flights',
	admin: 'Admin'
};

export const ROLE_OPTIONS = (Object.keys(ROLE_LABELS) as Role[]).map((value) => ({
	value,
	label: ROLE_LABELS[value]
}));

/** What each role may do, for help text next to the choice. */
export const ROLE_HELP =
	'Members see their own flights and their team’s. All flights sees every flight. Admins also manage users, pilots, teams and equipment.';
