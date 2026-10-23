import { fail } from '@sveltejs/kit';
import { get, send, type User } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	const archived = url.searchParams.has('archived');
	const users = await get<User[]>(fetch, `/api/users${archived ? '?archived=true' : ''}`);
	return { users, archived };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			roles: form.getAll('role').map(String),
			email: String(form.get('email') ?? '') || null
		};
		const result = await send<User>(fetch, 'POST', '/api/users', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { created: result.data.name };
	}
};
