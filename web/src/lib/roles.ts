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
	pilot: 'Flies for every group: appears in all pilot lists and pairs controllers.',
	editor: 'Sees and organises everything: sets pilots, archives flights, edits sessions.',
	admin: 'Manages users, groups, equipment and areas. Doesn’t fly without Pilot.'
};

/** What each role allows in a group. */
export const GROUP_ROLE_HELP: Record<GroupRole, string> = {
	viewer: 'Sees the group’s flights, sessions and equipment.',
	pilot: 'Flies for the group: appears in its pilot lists and pairs controllers.',
	editor: 'Sees and organises the group’s work: sets pilots, edits sessions.'
};

export const GROUP_ROLES: GroupRole[] = ['viewer', 'pilot', 'editor'];
export const ORGANISATION_ROLES: OrganisationRole[] = [...GROUP_ROLES, 'admin'];

/** Roles as a list for a cell: "Pilot, Editor". */
export function roleList(roles: OrganisationRole[]): string {
	return roles.map((role) => ROLE_LABELS[role]).join(', ');
}
