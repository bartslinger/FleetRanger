import { fail } from '@sveltejs/kit';
import { get, send, type User } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	return { user: await get<User>(fetch, `/api/accounts/${params.id}`) };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const role = String((await request.formData()).get('role') ?? '');
		const result = await send<User>(fetch, 'PUT', `/api/accounts/${params.id}`, { role });
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	},
	disable: async ({ fetch, params }) => {
		const result = await send<User>(fetch, 'POST', `/api/accounts/${params.id}/disable`);
		// Shown in the confirmation dialog, not on the page.
		if (!result.ok) return fail(result.status, { disableMessage: result.message });
		return { disabled: true };
	},
	enable: async ({ fetch, params }) => {
		const result = await send<User>(fetch, 'POST', `/api/accounts/${params.id}/enable`);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { enabled: true };
	}
};
