import { fail } from '@sveltejs/kit';
import { get, send, type User } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	const disabled = url.searchParams.has('disabled');
	const users = await get<User[]>(fetch, '/api/accounts');
	return { users: disabled ? users : users.filter((user) => !user.disabled_at), disabled };
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const email = String(form.get('email') ?? '');
		const role = String(form.get('role') ?? 'member');
		const result = await send<User>(fetch, 'POST', '/api/accounts', { email, role });
		if (!result.ok) return fail(result.status, { message: result.message });
		return { created: result.data.email };
	}
};
