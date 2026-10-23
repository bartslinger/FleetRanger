import { fail } from '@sveltejs/kit';
import { get, send, type Battery, type BatteryDetail } from '#lib/server/backend.js';

export async function load({ fetch, locals, params }) {
	const battery = await get<BatteryDetail>(fetch, `/api/batteries/${params.id}`);
	// Batteries have no group, so only admins name them.
	return { battery, edit: locals.account?.role === 'admin' };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = { name: String(form.get('name') ?? '') || null };
		const result = await send<Battery>(fetch, 'PUT', `/api/batteries/${params.id}`, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	}
};
