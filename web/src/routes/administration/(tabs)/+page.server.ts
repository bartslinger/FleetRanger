import { error, fail } from '@sveltejs/kit';
import { may } from '#lib/permissions.js';
import { get, send, type OrganisationSettings } from '#lib/server/backend.js';

export async function load({ fetch, locals }) {
	if (!may(locals.account, 'organisation.manage', null)) error(403);
	return { organisation: await get<OrganisationSettings>(fetch, '/api/organisation') };
}

export const actions = {
	save: async ({ fetch, request }) => {
		const form = await request.formData();
		const offered = form.has('mcp_offered');
		const body = {
			name: String(form.get('name') ?? ''),
			time_zone: String(form.get('time_zone') ?? ''),
			session_gap_minutes: Number(form.get('session_gap_minutes')),
			// Left as it is where the install doesn't offer MCP.
			mcp: offered ? form.has('mcp') : null
		};
		const result = await send<OrganisationSettings>(fetch, 'PUT', '/api/organisation', body);
		if (!result.ok) return fail(result.status, { message: result.message });
		return { saved: true };
	}
};
