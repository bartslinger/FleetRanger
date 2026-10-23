import { fail } from '@sveltejs/kit';
import { get, remove, send, type PairingCode } from '#lib/server/backend.js';

// Any user creates a single-use code here, which the pilot types into the
// controller; see docs/pairing.md.

export async function load({ fetch }) {
	return {
		// The creator's teams: the controller joins one, chosen if several.
		teams: await get<{ id: string; name: string }[]>(fetch, '/api/pairing/teams'),
		// Theirs, or everyone's for admins.
		codes: await get<PairingCode[]>(fetch, '/api/pairing-codes')
	};
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const team = String(form.get('team_id') ?? '');
		const hours = Number(form.get('hours'));
		const result = await send<{ code: string; team: string | null; expires_at: string }>(
			fetch,
			'POST',
			'/api/pairing-codes',
			{ team: team || null, hours }
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
