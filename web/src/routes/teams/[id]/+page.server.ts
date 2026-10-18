import { fail } from '@sveltejs/kit';
import { get, send, type Team } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	return { team: await get<Team>(fetch, `/api/teams/${params.id}`) };
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
		if (!result.ok) return fail(result.status, { message: result.message });
		return { archived: true };
	},
	restore: async ({ fetch, params }) => {
		const result = await send<Team>(fetch, 'POST', `/api/teams/${params.id}/restore`);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { restored: true };
	}
};
