import { fail } from '@sveltejs/kit';
import { find, send, type PairingRequest } from '#lib/server/backend.js';

// A controller shows a code and a QR code linking here with it (?code=…).

export async function load({ fetch, url }) {
	const code = url.searchParams.get('code')?.trim() ?? '';
	const request = code
		? await find<PairingRequest>(fetch, `/api/pairing/${encodeURIComponent(code)}`)
		: null;
	return { code, request };
}

export const actions = {
	approve: async ({ fetch, request }) => {
		const code = String((await request.formData()).get('code') ?? '');
		const result = await send<{ serial: string }>(
			fetch,
			'POST',
			`/api/pairing/${encodeURIComponent(code)}/approve`
		);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { paired: result.data.serial };
	}
};
