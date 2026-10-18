import { fail } from '@sveltejs/kit';
import { get, send, type Pilot, type Team, type User } from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [pilot, teams, users] = await Promise.all([
		get<Pilot>(fetch, `/api/pilots/${params.id}`),
		get<Team[]>(fetch, '/api/teams'),
		get<User[]>(fetch, '/api/accounts')
	]);
	// Users without a pilot, and the one linked to this pilot; disabled users
	// can't log in, so linking them makes no sense.
	const linkable = users.filter(
		(user) => user.id === pilot.account_id || (!user.pilot_id && !user.disabled_at)
	);
	return { pilot, teams, users: linkable };
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const body = {
			name: String(form.get('name') ?? ''),
			team_id: String(form.get('team_id') ?? ''),
			account_id: String(form.get('account_id') ?? '') || null
		};
		const result = await send<Pilot>(fetch, 'PUT', `/api/pilots/${params.id}`, body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	},
	archive: async ({ fetch, params }) => {
		const result = await send<Pilot>(fetch, 'POST', `/api/pilots/${params.id}/archive`);
		// Shown in the confirmation dialog, not on the page.
		if (!result.ok) return fail(result.status, { archiveMessage: result.message });
		return { archived: true };
	},
	restore: async ({ fetch, params }) => {
		const result = await send<Pilot>(fetch, 'POST', `/api/pilots/${params.id}/restore`);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { restored: true };
	}
};
