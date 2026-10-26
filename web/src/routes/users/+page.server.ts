import { fail } from '@sveltejs/kit';
import { get, send, type OfferedAppRole, type User } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	const archived = url.searchParams.has('archived');
	const users = await get<User[]>(fetch, `/api/users${archived ? '?archived=true' : ''}`);
	const appRoles = await get<OfferedAppRole[]>(fetch, '/api/apps/roles');
	return { users, archived, appRoles };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			roles: form.getAll('role').map(String),
			email: String(form.get('email') ?? '') || null,
			app_roles: form.getAll('app_role').map(String)
		};
		const result = await send<User>(fetch, 'POST', '/api/users', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { created: result.data.name };
	},
	/** A user's roles on the organisation, as on a group's members; name and email stay. */
	roles: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('user_id') ?? '');
		const user = await get<User>(fetch, `/api/users/${id}`);
		const body = {
			name: user.name,
			roles: form.getAll('role').map(String),
			email: user.email,
			app_roles: form.getAll('app_role').map(String)
		};
		const result = await send<User>(fetch, 'PUT', `/api/users/${id}`, body);
		if (!result.ok) return fail(result.status, { rolesMessage: result.message });
		return { saved: result.data.name };
	}
};
