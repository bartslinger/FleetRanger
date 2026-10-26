import { fail, redirect } from '@sveltejs/kit';
import {
	get,
	remove,
	send,
	type Group,
	type GroupDetail,
	type Member,
	type OfferedAppRole,
	type User
} from '#lib/server/backend.js';

export async function load({ fetch, params }) {
	const [group, users, groups, appRoles] = await Promise.all([
		get<GroupDetail>(fetch, `/api/groups/${params.id}`),
		get<User[]>(fetch, '/api/users'),
		get<Group[]>(fetch, '/api/groups'),
		get<OfferedAppRole[]>(fetch, '/api/apps/roles')
	]);
	return {
		group,
		appRoles,
		// Who can be added: active users not in it yet.
		candidates: users
			.filter((user) => !group.member_list.some((member) => member.user_id === user.id))
			.map((user) => ({ id: user.id, name: user.name })),
		// Where its contents can go when it's deleted.
		others: groups.filter((other) => other.id !== group.id).map(({ id, name }) => ({ id, name }))
	};
}

export const actions = {
	save: async ({ fetch, params, request }) => {
		const name = String((await request.formData()).get('name') ?? '');
		const result = await send<GroupDetail>(fetch, 'PUT', `/api/groups/${params.id}`, { name });
		if (!result.ok) return fail(result.status, { message: result.message, name });
		return { saved: true };
	},
	roles: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const user = String(form.get('user_id') ?? '');
		if (!user) return fail(400, { rolesMessage: 'Choose a user.' });
		const roles = form.getAll('role').map(String);
		const result = await send<Member[]>(fetch, 'PUT', `/api/groups/${params.id}/members/${user}`, {
			roles,
			app_roles: form.getAll('app_role').map(String)
		});
		if (!result.ok) return fail(result.status, { rolesMessage: result.message });
		return { member: result.data.find((member) => member.user_id === user)?.name };
	},
	removeMember: async ({ fetch, params, request }) => {
		const user = String((await request.formData()).get('user_id') ?? '');
		const result = await remove(fetch, `/api/groups/${params.id}/members/${user}`);
		if (!result.ok) return fail(result.status, { removeMessage: result.message });
		return { removed: true };
	},
	delete: async ({ fetch, params, request }) => {
		const form = await request.formData();
		const moveTo = String(form.get('move_to') ?? '');
		// A group that holds something asks where it goes.
		if (form.has('move_to') && !moveTo) {
			return fail(400, { moveError: 'Choose where to move everything first.' });
		}
		const query = moveTo ? `?move_to=${encodeURIComponent(moveTo)}` : '';
		const result = await remove(fetch, `/api/groups/${params.id}${query}`);
		if (!result.ok) return fail(result.status, { deleteMessage: result.message });
		redirect(303, '/groups');
	}
};
