import { fail } from '@sveltejs/kit';
import {
	get,
	remove,
	send,
	type Equipment,
	type FlightChoices,
	type Team,
	type User
} from '#lib/server/backend.js';
import { KINDS } from '#lib/equipment.js';

export async function load({ fetch, locals, params }) {
	const item = await get<Equipment>(fetch, `${KINDS[params.kind].api}/${params.id}`);
	const role = locals.account?.role;
	if (role === 'admin') {
		const [teams, users] = await Promise.all([
			get<Team[]>(fetch, '/api/teams'),
			get<User[]>(fetch, '/api/users')
		]);
		return { kind: params.kind, item, edit: 'all' as const, teams, holders: users };
	}
	if (role === 'member') {
		// Members choose a holder from the item's team: its pilots in the logbook.
		const choices = await get<FlightChoices>(fetch, '/api/flights/choices');
		const holders = choices.pilots.filter((pilot) => pilot.team_id === item.team_id);
		return { kind: params.kind, item, edit: 'name' as const, teams: [], holders };
	}
	return { kind: params.kind, item, edit: 'none' as const, teams: [], holders: [] };
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
