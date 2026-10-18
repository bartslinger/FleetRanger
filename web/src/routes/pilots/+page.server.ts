import { fail } from '@sveltejs/kit';
import { get, send, type Pilot, type Team } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	const archived = url.searchParams.has('archived');
	const [pilots, teams] = await Promise.all([
		get<Pilot[]>(fetch, `/api/pilots${archived ? '?archived=true' : ''}`),
		get<Team[]>(fetch, '/api/teams')
	]);
	return { pilots, teams, archived };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const name = String(form.get('name') ?? '');
		const team_id = String(form.get('team_id') ?? '');
		const result = await send<Pilot>(fetch, 'POST', '/api/pilots', { name, team_id });
		if (!result.ok) return fail(result.status, { message: result.message });
		return { created: result.data.name };
	}
};
