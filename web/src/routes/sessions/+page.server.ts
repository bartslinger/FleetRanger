import { fail, redirect } from '@sveltejs/kit';
import { get, send, type FlightChoices, type SessionPage } from '#lib/server/backend.js';

const PAGE_SIZES = [25, 50, 100];
const DEFAULT_SIZE = 50;

export async function load({ fetch, url }) {
	const page = Math.max(1, Number.parseInt(url.searchParams.get('page') ?? '', 10) || 1);
	const asked = Number.parseInt(url.searchParams.get('size') ?? '', 10);
	const size = PAGE_SIZES.includes(asked) ? asked : DEFAULT_SIZE;
	const api = new URLSearchParams({ page: String(page), size: String(size) });
	const [{ sessions, total }, choices] = await Promise.all([
		get<SessionPage>(fetch, `/api/sessions?${api}`),
		get<FlightChoices>(fetch, '/api/flights/choices')
	]);

	// Past the last page, e.g. after sessions were merged: go to the last one.
	const pages = Math.max(1, Math.ceil(total / size));
	if (page > pages) redirect(307, `${url.pathname}?page=${pages}&size=${size}`);

	return { sessions, total, page, size, sizes: PAGE_SIZES, choices };
}

export const actions = {
	merge: async ({ fetch, request }) => {
		const form = await request.formData();
		const body = { session_ids: form.getAll('session_id').map(String) };
		const result = await send<{ id: string }>(fetch, 'POST', '/api/sessions/merge', body);
		if (!result.ok) return fail(result.status, { mergeMessage: result.message });
		return { merged: result.data.id };
	}
};
