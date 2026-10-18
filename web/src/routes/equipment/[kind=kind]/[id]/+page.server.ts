import { fail } from '@sveltejs/kit';
import { get, send, type Equipment, type Pilot, type Team } from '#lib/server/backend.js';
import { KINDS } from '#lib/equipment.js';

export async function load({ fetch, params }) {
	const [item, teams, pilots] = await Promise.all([
		get<Equipment>(fetch, `${KINDS[params.kind].api}/${params.id}`),
		get<Team[]>(fetch, '/api/teams'),
		get<Pilot[]>(fetch, '/api/pilots')
	]);
	return { kind: params.kind, item, teams, pilots };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? '') || null,
			team_id: String(form.get('team_id') ?? '') || null,
			holder_pilot_id: String(form.get('holder_pilot_id') ?? '') || null
		};
		const path = `${KINDS[params.kind].api}/${params.id}`;
		const result = await send<Equipment>(fetch, 'PUT', path, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	}
};
