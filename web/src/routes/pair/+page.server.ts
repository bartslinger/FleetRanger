import { fail } from '@sveltejs/kit';
import { send } from '#lib/server/backend.js';

// A controller shows a code, and a QR code linking here with it (?code=…).
// Entering the code pairs the controller: the code identifies it.

export function load({ url }) {
	return { code: url.searchParams.get('code')?.trim() ?? '' };
}

export const actions = {
	pair: async ({ fetch, request }) => {
		const code = String((await request.formData()).get('code') ?? '').trim();
		const result = await send<{ serial: string; model: string }>(
			fetch,
			'POST',
			`/api/pairing/${encodeURIComponent(code)}/approve`
		);
		if (!result.ok) return fail(result.status, { message: result.message, code });
		return { paired: result.data };
	}
};
