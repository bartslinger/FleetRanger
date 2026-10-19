import { fail } from '@sveltejs/kit';
import { get, send } from '#lib/server/backend.js';

// A controller shows a code, and a QR code linking here with it (?code=…).
// Entering the code pairs the controller: the code identifies it.

export async function load({ fetch, url }) {
	return {
		code: url.searchParams.get('code')?.trim() ?? '',
		// The approver's teams: the controller joins one, chosen if several.
		teams: await get<{ id: string; name: string }[]>(fetch, '/api/pairing/teams')
	};
}

export const actions = {
	pair: async ({ fetch, request }) => {
		const form = await request.formData();
		const code = String(form.get('code') ?? '').trim();
		const team = String(form.get('team_id') ?? '');
		const result = await send<{ serial: string; model: string }>(
			fetch,
			'POST',
			`/api/pairing/${encodeURIComponent(code)}/approve${team ? `?team=${encodeURIComponent(team)}` : ''}`
		);
		if (!result.ok) return fail(result.status, { message: result.message, code });
		return { paired: result.data };
	}
};
