import { fail } from '@sveltejs/kit';
import { get, send, type Group, type GroupDetail } from '#lib/server/backend.js';

export async function load({ fetch }) {
	return { groups: await get<Group[]>(fetch, '/api/groups') };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const name = String((await request.formData()).get('name') ?? '');
		const result = await send<GroupDetail>(fetch, 'POST', '/api/groups', { name });
		if (!result.ok) return fail(result.status, { message: result.message, name });
		return { created: result.data.name };
	}
};
