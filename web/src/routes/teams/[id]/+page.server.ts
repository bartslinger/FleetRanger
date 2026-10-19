import { fail } from '@sveltejs/kit';
import { get, send, type Team, type User } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [team, users] = await Promise.all([
		get<Team>(fetch, `/api/teams/${params.id}`),
		get<User[]>(fetch, '/api/users')
	]);
	return { team, members: users.filter((user) => user.team_id === team.id) };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const name = String((await request.formData()).get('name') ?? '');
		const result = await send<Team>(fetch, 'PUT', `/api/teams/${params.id}`, { name });
		if (!result.ok) return fail(result.status, { message: result.message, name });
		return { saved: true };
	},
	archive: async ({ fetch, params }) => {
		const result = await send<Team>(fetch, 'POST', `/api/teams/${params.id}/archive`);
		// Shown in the confirmation dialog, not on the page.
		if (!result.ok) return fail(result.status, { archiveMessage: result.message });
		return { archived: true };
	},
	restore: async ({ fetch, params }) => {
		const result = await send<Team>(fetch, 'POST', `/api/teams/${params.id}/restore`);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { restored: true };
	}
};
