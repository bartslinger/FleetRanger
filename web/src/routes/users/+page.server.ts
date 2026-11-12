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
	}
};
