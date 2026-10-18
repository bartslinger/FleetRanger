import type { Handle } from '@sveltejs/kit/hooks';
import { parseTheme, THEME_COOKIE } from '#lib/theme.js';

// Renders the user's theme choice on <html>, so the page never shows the
// other theme first.
export const handle: Handle = async ({ event, resolve }) => {
	const theme = parseTheme(event.cookies.get(THEME_COOKIE));
	event.locals.theme = theme;
	return resolve(event, {
		transformPageChunk: ({ html }) =>
			html.replace('%fleetranger.theme%', theme === 'system' ? '' : `data-theme="${theme}"`)
	});
};
