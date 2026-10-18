import { get, type Download } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	return {
		download: await get<Download>(fetch, '/api/download'),
		// This install, as the browser reached it.
		server: url.origin,
		// What to type in the app, which adds https:// itself (a dev server is http).
		address: url.protocol === 'https:' ? url.host : url.origin
	};
}
