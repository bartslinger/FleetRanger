// A user's roles as the roles dialogs post them. Admin is a role on the
// organisation; Viewer, Pilot and Editor and the apps' roles are given in
// groups. While the organisation has only one group, the dialogs show them
// all together and the others go to that group (user's decision,
// 2026-11-11: no groups shown until there are several).

import { get, remove, send, type Group, type SendResult, type User } from '#lib/server/backend.js';

/** The organisation's only group, or null when it has several. */
export async function onlyGroup(fetch: typeof globalThis.fetch): Promise<Group | null> {
	const groups = await get<Group[]>(fetch, '/api/groups');
	return groups.length === 1 ? groups[0] : null;
}

/** The posted roles, split into the organisation's and a group's. */
export function postedRoles(form: FormData) {
	const roles = form.getAll('role').map(String);
	return {
		organisation: roles.filter((role) => role === 'admin'),
		group: roles.filter((role) => role !== 'admin'),
		appRoles: form.getAll('app_role').map(String)
	};
}

/**
 * Saves a user's roles in the only group: their membership with these
 * roles, or none without any.
 *
 * - `user`: as it was before, to know whether they were a member.
 */
export async function saveInOnlyGroup(
	fetch: typeof globalThis.fetch,
	group: Group,
	user: Pick<User, 'id' | 'groups'>,
	roles: string[],
	appRoles: string[]
): Promise<SendResult<unknown>> {
	const path = `/api/groups/${group.id}/members/${user.id}`;
	if (roles.length > 0 || appRoles.length > 0) {
		return send(fetch, 'PUT', path, { roles, app_roles: appRoles });
	}
	if (user.groups.some((membership) => membership.id === group.id)) return remove(fetch, path);
	return { ok: true, data: null };
}
