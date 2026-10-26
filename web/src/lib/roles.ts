// Roles as the web app shows them; see "Groups and roles" in
// docs/architecture.md.

import type { GroupRole, OfferedAppRole, OrganisationRole } from '#lib/server/backend.js';

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

/** The apps' roles grouped by app, in the order given (by app, then name). */
export function byApp(appRoles: OfferedAppRole[]) {
	const apps: { id: string; name: string; roles: OfferedAppRole[] }[] = [];
	for (const role of appRoles) {
		const last = apps.at(-1);
		if (last?.id === role.app_id) last.roles.push(role);
		else apps.push({ id: role.app_id, name: role.app, roles: [role] });
	}
	return apps;
}

/** FleetRanger's roles and then each app's, for a cell:
 * "Pilot, Editor · Reports: Report viewer" (carbon-expert's advice). */
export function allRoles(
	roles: OrganisationRole[],
	appRoleIds: string[],
	appRoles: OfferedAppRole[]
): string {
	const held = appRoles.filter((role) => appRoleIds.includes(role.id));
	return [
		roleList(roles),
		...byApp(held).map((app) => `${app.name}: ${app.roles.map((role) => role.name).join(', ')}`)
	]
		.filter(Boolean)
		.join(' · ');
}
