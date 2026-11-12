import { fail, redirect } from '@sveltejs/kit';
import {
	get,
	remove,
	send,
	type Group,
	type Member,
	type OfferedAppRole,
	type User
} from '#lib/server/backend.js';
import { onlyGroup, postedRoles, saveInOnlyGroup } from '#lib/server/userRoles.js';

export async function load({ fetch, params }) {
	const [user, users, groups, appRoles] = await Promise.all([
		get<User>(fetch, `/api/users/${params.id}`),
		get<User[]>(fetch, '/api/users'),
		get<Group[]>(fetch, '/api/groups'),
		get<OfferedAppRole[]>(fetch, '/api/apps/roles')
	]);
	// The server keeps the last active admin; the page explains it.
	const admins = users.filter((other) => other.roles.includes('admin'));
	const lastAdmin = user.archived_at === null && admins.length === 1 && admins[0].id === user.id;
	// The groups they can be added to: those they aren't in yet.
	const otherGroups = groups
		.filter((group) => !user.groups.some((own) => own.id === group.id))
		.map(({ id, name }) => ({ id, name }));
	// While there is only one group, its roles are shown with Admin and no
	// groups are shown.
	const onlyGroup = groups.length === 1 ? groups[0].id : null;
	return { user, lastAdmin, otherGroups, appRoles, onlyGroup };
}

export const actions = {
	/** Name, email and Admin; while there is only one group, also the roles
	 * in it. */
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const user = await get<User>(fetch, `/api/users/${params.id}`);
		const roles = postedRoles(form);
		const body = {
			name: String(form.get('name') ?? ''),
			roles: roles.organisation,
			email: String(form.get('email') ?? '') || null
		};
		const result = await send<User>(fetch, 'PUT', `/api/users/${params.id}`, body);
		if (!result.ok) return fail(result.status, { editMessage: result.message });
		const only = await onlyGroup(fetch);
		if (only) {
			const saved = await saveInOnlyGroup(fetch, only, user, roles.group, roles.appRoles);
			if (!saved.ok) return fail(saved.status, { editMessage: saved.message });
		}
		return { saved: true };
	},
	/** Adds them to a group or changes their roles in it, as on the group's page. */
	groupRoles: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const group = String(form.get('group_id') ?? '');
		if (!group) return fail(400, { groupRolesMessage: 'Choose a group.' });
		const roles = form.getAll('role').map(String);
		const appRoles = form.getAll('app_role').map(String);
		const result = await send<Member[]>(fetch, 'PUT', `/api/groups/${group}/members/${params.id}`, {
			roles,
			app_roles: appRoles
		});
		if (!result.ok) return fail(result.status, { groupRolesMessage: result.message });
		return { groupSaved: true };
	},
	leaveGroup: async ({ fetch, params, request }) => {
		const group = String((await request.formData()).get('group_id') ?? '');
		const result = await remove(fetch, `/api/groups/${group}/members/${params.id}`);
		if (!result.ok) return fail(result.status, { leaveMessage: result.message });
		return { left: true };
	},
	archive: async ({ fetch, params }) => {
		const result = await send<User>(fetch, 'POST', `/api/users/${params.id}/archive`);
		// Shown in the confirmation dialog, not on the page.
		if (!result.ok) return fail(result.status, { archiveMessage: result.message });
		return { archived: true };
	},
	restore: async ({ fetch, params }) => {
		const result = await send<User>(fetch, 'POST', `/api/users/${params.id}/restore`);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { restored: true };
	},
	delete: async ({ fetch, params }) => {
		const result = await remove(fetch, `/api/users/${params.id}`);
		if (!result.ok) return fail(result.status, { deleteMessage: result.message });
		redirect(303, '/users');
	}
};
