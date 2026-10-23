// Roles as the web app shows them; see "Groups and roles" in
// docs/architecture.md.

import type { GroupRole, OrganisationRole } from '#lib/server/backend.js';

export const ROLE_LABELS: Record<OrganisationRole, string> = {
	viewer: 'Viewer',
	pilot: 'Pilot',
	editor: 'Editor',
	admin: 'Admin'
};

/** What each role allows, for helper text beside the choice. */
export const ROLE_HELP: Record<OrganisationRole, string> = {
	viewer: 'Sees the flights, sessions and equipment.',
	pilot: 'Flies: appears in pilot lists and pairs controllers. Sees their own flights.',
	editor: 'Sees and organises: sets pilots, archives flights, edits sessions, names equipment.',
	admin: 'Manages users, groups, equipment, areas and log processing. Doesn’t fly.'
};

export const GROUP_ROLES: GroupRole[] = ['viewer', 'pilot', 'editor'];
export const ORGANISATION_ROLES: OrganisationRole[] = [...GROUP_ROLES, 'admin'];

/** Roles as a list for a cell: "Pilot, Editor". */
export function roleList(roles: OrganisationRole[]): string {
	return roles.map((role) => ROLE_LABELS[role]).join(', ');
}
