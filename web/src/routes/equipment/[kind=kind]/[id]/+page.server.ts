import { fail } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
import { get, remove, send, type Equipment, type FlightChoices } from '#lib/server/backend.js';
import { KINDS } from '#lib/equipment.js';

export async function load({ fetch, locals, params }) {
	const item = await get<Equipment>(fetch, `${KINDS[params.kind].api}/${params.id}`);
	// Renaming needs equipment.rename on its group, moving equipment.move on
	// both groups: offered are the groups where the user may move it to, once
	// there are several.
	const rename = may(locals.account, 'equipment.rename', item.group_id);
	const move =
		(locals.account?.several_groups ?? false) &&
		may(locals.account, 'equipment.move', item.group_id);
	const groups = move
		? (await get<FlightChoices>(fetch, '/api/flights/choices')).groups.filter((group) =>
				may(locals.account, 'equipment.move', group.id)
			)
		: [];
	return {
		kind: params.kind,
		item,
		rename,
		move,
		groups,
		showsGroups: locals.account?.several_groups ?? false,
		unpair: may(locals.account, 'controllers.manage', item.group_id)
	};
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? '') || null,
			group_id: String(form.get('group_id') ?? '')
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
