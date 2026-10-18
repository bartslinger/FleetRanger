import { fail } from '@sveltejs/kit';
import { get, remove, send, type Equipment, type Team, type User } from '#lib/server/backend.js';
import { KINDS } from '#lib/equipment.js';

export async function load({ fetch, params }) {
	const [item, teams, users] = await Promise.all([
		get<Equipment>(fetch, `${KINDS[params.kind].api}/${params.id}`),
		get<Team[]>(fetch, '/api/teams'),
		get<User[]>(fetch, '/api/users')
	]);
	return { kind: params.kind, item, teams, users };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? '') || null,
			team_id: String(form.get('team_id') ?? '') || null,
			holder_user_id: String(form.get('holder_user_id') ?? '') || null
		};
		const path = `${KINDS[params.kind].api}/${params.id}`;
		const result = await send<Equipment>(fetch, 'PUT', path, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	},
	unpair: async ({ fetch, params }) => {
		const result = await remove(fetch, `/api/remote-controllers/${params.id}/pairing`);
		// Shown in the confirmation dialog, not on the page.
		if (!result.ok) return fail(result.status, { unpairMessage: result.message });
		return { unpaired: true };
	}
};
