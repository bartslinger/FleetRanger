// Roles as the web app shows them; see "Groups and roles" in
// docs/architecture.md.

import type { GroupRole, OrganisationRole } from '#lib/server/backend.js';

export const ROLE_LABELS: Record<OrganisationRole, string> = {
	viewer: 'Viewer',
	pilot: 'Pilot',
	editor: 'Editor',
	admin: 'Admin'
};

/** What each role allows on the whole organisation, for helper text. */
export const ROLE_HELP: Record<OrganisationRole, string> = {
	viewer: 'Sees all flights, sessions and equipment.',
	pilot: 'Flies for every group.',
	editor: 'Sees and organises all flights and sessions.',
	admin: 'Manages users, groups and equipment. Doesn’t fly.'
};

/** What each role allows in a group. */
export const GROUP_ROLE_HELP: Record<GroupRole, string> = {
	viewer: 'Sees the group’s flights, sessions and equipment.',
	pilot: 'Flies for the group.',
	editor: 'Sees and organises the group’s flights and sessions.'
};

export const GROUP_ROLES: GroupRole[] = ['viewer', 'pilot', 'editor'];
export const ORGANISATION_ROLES: OrganisationRole[] = [...GROUP_ROLES, 'admin'];

/** Roles as a list for a cell: "Pilot, Editor". */
export function roleList(roles: OrganisationRole[]): string {
	return roles.map((role) => ROLE_LABELS[role]).join(', ');
}
