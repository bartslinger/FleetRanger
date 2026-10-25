import { error, fail, redirect } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
import { get, remove, send, type TrustedApp } from '#lib/server/backend.js';
import { addressLines } from '../addresses.js';

export async function load({ fetch, locals, params }) {
	if (!locals.account?.apps || !may(locals.account, 'apps.manage', null)) error(403);
	// An install has a handful of apps: the list holds this one too.
	const apps = await get<TrustedApp[]>(fetch, '/api/apps');
	const app = apps.find((app) => app.id === params.id);
	if (!app) error(404);
	return { app };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			redirect_uris: addressLines(String(form.get('redirect_uris') ?? ''))
		};
		const result = await send<null>(fetch, 'PUT', `/api/apps/${params.id}`, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	},
	delete: async ({ fetch, params }) => {
		const result = await remove(fetch, `/api/apps/${params.id}`);
		if (!result.ok) return fail(result.status, { deleteMessage: result.message });
		redirect(303, '/apps');
	}
};
