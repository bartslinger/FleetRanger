import { fail } from '@sveltejs/kit';
import { get, remove, send, type PairingCode } from '#lib/server/backend.js';

// Whoever holds controllers.pair creates a single-use code here, which the
// pilot types into the controller; see "Clients" in docs/architecture.md.

export async function load({ fetch }) {
	return {
		// Where the controller can go: the teams where the user may pair, and
		// the organisation itself when they may pair there.
		destinations: await get<{ teams: { id: string; name: string }[]; organisation: boolean }>(
			fetch,
			'/api/pairing-codes/teams'
		),
		// Theirs, and those of the teams whose controllers they manage.
		codes: await get<PairingCode[]>(fetch, '/api/pairing-codes')
	};
}

export const actions = {
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const team = String(form.get('team_id') ?? '');
		if (!team) return fail(400, { teamError: 'Choose a team for the controller.' });
		const hours = Number(form.get('hours'));
		const result = await send<{ code: string; team: string | null; expires_at: string }>(
			fetch,
			'POST',
			'/api/pairing-codes',
			// "organisation": no team, as the select's empty value is its prompt.
			{ team: team === 'organisation' ? null : team, hours }
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
