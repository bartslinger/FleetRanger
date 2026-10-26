import { error, fail, redirect } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
import { get, remove, send, type TrustedApp } from '#lib/server/backend.js';
import { addressLines } from '../addresses.js';

/** One app from the list: an install has a handful, so the list holds it. */
async function find(fetch: typeof globalThis.fetch, id: string) {
	const apps = await get<TrustedApp[]>(fetch, '/api/apps');
	return apps.find((app) => app.id === id);
}

/** The app's roles as the backend takes them back. */
const asInput = (app: TrustedApp) => app.roles.map(({ id, key, name }) => ({ id, key, name }));

export async function load({ fetch, locals, params }) {
	if (!locals.account?.apps || !may(locals.account, 'apps.manage', null)) error(403);
	const app = await find(fetch, params.id);
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
	/** Adds a role, or with an id changes one; the app keeps the others. */
	role: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const app = await find(fetch, params.id);
		if (!app) return fail(404, { roleMessage: 'This app no longer exists.' });
		const id = String(form.get('id') ?? '') || undefined;
		const role = { id, key: String(form.get('key') ?? ''), name: String(form.get('name') ?? '') };
		const roles = id
			? asInput(app).map((other) => (other.id === id ? role : other))
			: [...asInput(app), role];
		const body = { name: app.name, redirect_uris: app.redirect_uris, roles };
		const result = await send<null>(fetch, 'PUT', `/api/apps/${params.id}`, body);
		if (!result.ok) return fail(result.status, { roleMessage: result.message });
		return { saved: true };
	},
	deleteRole: async ({ fetch, params, request }) => {
		const id = String((await request.formData()).get('id') ?? '');
		const app = await find(fetch, params.id);
		if (!app) return fail(404, { deleteRoleMessage: 'This app no longer exists.' });
		const roles = asInput(app).filter((role) => role.id !== id);
		const body = { name: app.name, redirect_uris: app.redirect_uris, roles };
		const result = await send<null>(fetch, 'PUT', `/api/apps/${params.id}`, body);
		if (!result.ok) return fail(result.status, { deleteRoleMessage: result.message });
		return { deleted: true };
	},
	delete: async ({ fetch, params }) => {
		const result = await remove(fetch, `/api/apps/${params.id}`);
		if (!result.ok) return fail(result.status, { deleteMessage: result.message });
		redirect(303, '/apps');
	}
};
