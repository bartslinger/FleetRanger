import { fail } from '@sveltejs/kit';
import {
	get,
	remove,
	send,
	type Equipment,
	type FlightChoices,
	type Group
} from '#lib/server/backend.js';
import { KINDS } from '#lib/equipment.js';

export async function load({ fetch, locals, params }) {
	const item = await get<Equipment>(fetch, `${KINDS[params.kind].api}/${params.id}`);
	const role = locals.account?.role;
	if (role === 'admin') {
		const teams = await get<Group[]>(fetch, '/api/groups');
		return { kind: params.kind, item, edit: 'all' as const, teams };
	}
	if (role === 'member' && item.team_id) {
		// Members rename their own teams' equipment, not another team's they
		// see through its flights: the teams whose flights they change, as
		// editors do, until the web app asks for permissions.
		const choices = await get<FlightChoices>(fetch, '/api/flights/choices');
		if (choices.own_team_ids.includes(item.team_id)) {
			return { kind: params.kind, item, edit: 'name' as const, teams: [] };
		}
	}
	return { kind: params.kind, item, edit: 'none' as const, teams: [] };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? '') || null,
			team_id: String(form.get('team_id') ?? '') || null
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
