import { fail } from '@sveltejs/kit';
import { get, remove, send, type PairingCode } from '#lib/server/backend.js';

// Whoever holds controllers.pair creates a single-use code here, which the
// pilot types into the controller; see "Clients" in docs/architecture.md.

export async function load({ fetch }) {
	return {
		// Where the controller can go: the groups where the user may pair, and
		// the organisation itself when they may pair there.
		destinations: await get<{ groups: { id: string; name: string }[]; organisation: boolean }>(
			fetch,
			'/api/pairing-codes/groups'
		),
		// Theirs, and those of the groups whose controllers they manage.
		codes: await get<PairingCode[]>(fetch, '/api/pairing-codes')
	};
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const group = String(form.get('group_id') ?? '');
		if (!group) return fail(400, { groupError: 'Choose a group for the controller.' });
		const hours = Number(form.get('hours'));
		const result = await send<{ code: string; group: string | null; expires_at: string }>(
			fetch,
			'POST',
			'/api/pairing-codes',
			// "organisation": no group, as the select's empty value is its prompt.
			{ group: group === 'organisation' ? null : group, hours }
		);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { created: result.data };
	},
	revoke: async ({ fetch, request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const result = await remove(fetch, `/api/pairing-codes/${encodeURIComponent(id)}`);
		if (!result.ok) return fail(result.status, { revokeMessage: result.message });
		return { revoked: String(form.get('code') ?? '') };
	}
};
