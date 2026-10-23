import { fail, redirect } from '@sveltejs/kit';
import { get, remove, send, type User } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [user, users] = await Promise.all([
		get<User>(fetch, `/api/users/${params.id}`),
		get<User[]>(fetch, '/api/users')
	]);
	// The server keeps the last active admin; the page explains it.
	const admins = users.filter((other) => other.roles.includes('admin'));
	const lastAdmin = user.archived_at === null && admins.length === 1 && admins[0].id === user.id;
	return { user, lastAdmin };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			roles: form.getAll('role').map(String),
			email: String(form.get('email') ?? '') || null
		};
		const result = await send<User>(fetch, 'PUT', `/api/users/${params.id}`, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
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
