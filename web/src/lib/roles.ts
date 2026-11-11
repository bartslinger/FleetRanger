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
	pilot: 'Flies for the organisation.',
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

/** FleetRanger's roles and then the apps', by name only, for a cell:
 * "Pilot, Editor, Report creator" (user's wish, 2026-11-11). */
export function allRoles(
	roles: OrganisationRole[],
	appRoleIds: string[],
	appRoles: OfferedAppRole[]
): string {
	return roleNames(roles, appRoleIds, appRoles).join(', ');
}

function roleNames(
	roles: OrganisationRole[],
	appRoleIds: string[],
	appRoles: OfferedAppRole[]
): string[] {
	return [
		...roles.map((role) => ROLE_LABELS[role]),
		...appRoles.filter((role) => appRoleIds.includes(role.id)).map((role) => role.name)
	];
}

/**
 * A user's roles over several groups, per role: "Admin; Pilot, Editor in 4
 * groups; Viewer in South". Roles held in the same groups go together.
 */
export function rolesAcrossGroups(
	admin: boolean,
	groups: { name: string; roles: OrganisationRole[]; app_roles: string[] }[],
	appRoles: OfferedAppRole[]
): string {
	// Each role's groups, in the order roles are listed.
	const where = new Map<string, string[]>();
	for (const group of groups) {
		for (const name of roleNames(group.roles, group.app_roles, appRoles)) {
			where.set(name, [...(where.get(name) ?? []), group.name]);
		}
	}
	// Roles with the same groups share one "in …".
	const together = new Map<string, string[]>();
	for (const [name, held] of where) {
		const place = held.length === 1 ? held[0] : `${held.length} groups`;
		const key = `${held.join('\n')}`;
		const entry = together.get(key);
		if (entry) entry.push(name);
		else together.set(key, [place, name]);
	}
	return [
		...(admin ? ['Admin'] : []),
		...[...together.values()].map(([place, ...names]) => `${names.join(', ')} in ${place}`)
	].join('; ');
}
