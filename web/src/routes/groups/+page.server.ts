import { fail } from '@sveltejs/kit';
import { get, send, type Group, type GroupDetail, type User } from '#lib/server/backend.js';

export async function load({ fetch }) {
	const [groups, users, unassigned] = await Promise.all([
		get<Group[]>(fetch, '/api/groups'),
		get<User[]>(fetch, '/api/users'),
		get<{ total: number }>(fetch, '/api/flights?group=none&size=1')
	]);
	// The organisation, shown as the top group: all users are its members,
	// and it holds the flights in no group.
	const organisation = { members: users.length, flights: unassigned.total };
	return { groups, organisation };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const name = String((await request.formData()).get('name') ?? '');
		const result = await send<GroupDetail>(fetch, 'POST', '/api/groups', { name });
		if (!result.ok) return fail(result.status, { message: result.message, name });
		return { created: result.data.name };
	}
};
