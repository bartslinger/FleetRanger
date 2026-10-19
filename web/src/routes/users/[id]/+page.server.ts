import { fail, redirect } from '@sveltejs/kit';
import { get, remove, send, type Team, type User } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [user, teams] = await Promise.all([
		get<User>(fetch, `/api/users/${params.id}`),
		get<Team[]>(fetch, '/api/teams')
	]);
	return { user, teams };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			role: String(form.get('role') ?? ''),
			team_ids: form.getAll('team_id').map(String),
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
