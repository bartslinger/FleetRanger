import { fail } from '@sveltejs/kit';
import { get, send, type Team } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	const archived = url.searchParams.has('archived');
	const teams = await get<Team[]>(fetch, `/api/teams${archived ? '?archived=true' : ''}`);
	return { teams, archived };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const name = String((await request.formData()).get('name') ?? '');
		const result = await send<Team>(fetch, 'POST', '/api/teams', { name });
		if (!result.ok) return fail(result.status, { message: result.message, name });
		return { created: result.data.name };
	}
};
