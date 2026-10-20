import { fail } from '@sveltejs/kit';
import { get, remove, type Connection } from '#lib/server/backend.js';

export async function load({ fetch }) {
	return { connections: await get<Connection[]>(fetch, '/api/oauth/connections') };
}

export const actions = {
	/** Disconnects an app: its tokens stop working at once. */
	disconnect: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const result = await remove(fetch, `/api/oauth/connections/${encodeURIComponent(id)}`);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { disconnected: String(form.get('app') ?? 'The app') };
	}
};
