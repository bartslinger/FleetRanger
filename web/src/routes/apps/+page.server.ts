import { error, fail, redirect } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
import { get, send, type TrustedApp } from '#lib/server/backend.js';
import { addressLines } from './addresses.js';

export async function load({ fetch, locals }) {
	// Only while apps can connect, and for who may add them.
	if (!locals.account?.apps || !may(locals.account, 'apps.manage', null)) error(403);
	return { apps: await get<TrustedApp[]>(fetch, '/api/apps') };
}

export const actions = {
	/** Adds an app, then opens its page, where its client ID is to copy. */
	create: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			redirect_uris: addressLines(String(form.get('redirect_uris') ?? ''))
		};
		const result = await send<{ id: string }>(fetch, 'POST', '/api/apps', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		redirect(303, `/apps/${result.data.id}`);
	}
};
