import { fail, redirect } from '@sveltejs/kit';
import { get, remove, send, type Group, type Member, type User } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [user, users, groups] = await Promise.all([
		get<User>(fetch, `/api/users/${params.id}`),
		get<User[]>(fetch, '/api/users'),
		get<Group[]>(fetch, '/api/groups')
	]);
	// The server keeps the last active admin; the page explains it.
	const admins = users.filter((other) => other.roles.includes('admin'));
	const lastAdmin = user.archived_at === null && admins.length === 1 && admins[0].id === user.id;
	// The groups they can be added to: those they aren't in yet.
	const otherGroups = groups
		.filter((group) => !user.groups.some((own) => own.id === group.id))
		.map(({ id, name }) => ({ id, name }));
	return { user, lastAdmin, otherGroups };
}

export const actions = {
	/** Name and email; the API takes the roles on the organisation along, as they are. */
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const user = await get<User>(fetch, `/api/users/${params.id}`);
		const body = {
			name: String(form.get('name') ?? ''),
			roles: user.roles,
			email: String(form.get('email') ?? '') || null
		};
		const result = await send<User>(fetch, 'PUT', `/api/users/${params.id}`, body);
		if (!result.ok) return fail(result.status, { editMessage: result.message });
		return { saved: true };
	},
	/** The roles on the organisation, edited as a group's are; name and email stay. */
	organisationRoles: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const user = await get<User>(fetch, `/api/users/${params.id}`);
		const body = { name: user.name, roles: form.getAll('role').map(String), email: user.email };
		const result = await send<User>(fetch, 'PUT', `/api/users/${params.id}`, body);
		if (!result.ok) return fail(result.status, { organisationRolesMessage: result.message });
		return { organisationSaved: true };
	},
	/** Adds them to a group or changes their roles in it, as on the group's page. */
	groupRoles: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const group = String(form.get('group_id') ?? '');
		if (!group) return fail(400, { groupRolesMessage: 'Choose a group.' });
		const roles = form.getAll('role').map(String);
		const result = await send<Member[]>(fetch, 'PUT', `/api/groups/${group}/members/${params.id}`, {
			roles
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
