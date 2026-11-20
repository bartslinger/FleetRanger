import { error } from '@sveltejs/kit';
import { find, sections } from '#lib/docs.js';
import type { EntryGenerator, PageLoad } from './$types';

export const entries: EntryGenerator = () =>
	sections.flatMap((s) => s.pages.map((p) => ({ section: s.slug, page: p.slug })));

export const load: PageLoad = ({ params }) => {
	const page = find(params.section, params.page);
	if (!page) error(404, 'Not found');
	return { page };
};
