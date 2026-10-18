import { fail } from '@sveltejs/kit';
import { get, send, type Team, type User } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	const archived = url.searchParams.has('archived');
	const [users, teams] = await Promise.all([
		get<User[]>(fetch, `/api/users${archived ? '?archived=true' : ''}`),
		get<Team[]>(fetch, '/api/teams')
	]);
	return { users, teams, archived };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			role: String(form.get('role') ?? 'member'),
			team_id: String(form.get('team_id') ?? '') || null,
			email: String(form.get('email') ?? '') || null
		};
		const result = await send<User>(fetch, 'POST', '/api/users', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { created: result.data.name };
	}
};
