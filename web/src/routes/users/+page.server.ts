import { fail } from '@sveltejs/kit';
import { get, send, type OfferedAppRole, type User } from '#lib/server/backend.js';
import { onlyGroup, postedRoles, saveInOnlyGroup } from '#lib/server/userRoles.js';

export async function load({ fetch, url }) {
	const archived = url.searchParams.has('archived');
	const users = await get<User[]>(fetch, `/api/users${archived ? '?archived=true' : ''}`);
	const appRoles = await get<OfferedAppRole[]>(fetch, '/api/apps/roles');
	const only = await onlyGroup(fetch);
	return { users, archived, appRoles, onlyGroup: only?.id ?? null };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const roles = postedRoles(form);
		const body = {
			name: String(form.get('name') ?? ''),
			roles: roles.organisation,
			email: String(form.get('email') ?? '') || null
		};
		const result = await send<User>(fetch, 'POST', '/api/users', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		const only = await onlyGroup(fetch);
		if (only) {
			const saved = await saveInOnlyGroup(fetch, only, result.data, roles.group, roles.appRoles);
			if (!saved.ok) return fail(saved.status, { message: saved.message });
		}
		return { created: result.data.name };
	},
	/** A user's roles, as on a group's members; name and email stay. With
	 * several groups only Admin is set here, the rest in the groups. */
	roles: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('user_id') ?? '');
		const user = await get<User>(fetch, `/api/users/${id}`);
		const roles = postedRoles(form);
		const only = await onlyGroup(fetch);
		// Two changes: the roles in the only group, then Admin on the organisation.
		if (only) {
			const saved = await saveInOnlyGroup(fetch, only, user, roles.group, roles.appRoles);
			if (!saved.ok) return fail(saved.status, { rolesMessage: saved.message });
		}
		const body = { name: user.name, roles: roles.organisation, email: user.email };
		const result = await send<User>(fetch, 'PUT', `/api/users/${id}`, body);
		if (!result.ok) return fail(result.status, { rolesMessage: result.message });
		return { saved: result.data.name };
	}
};
