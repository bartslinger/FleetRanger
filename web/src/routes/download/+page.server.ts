import { get, type Download } from '#lib/server/backend.js';

export async function load({ fetch, url }) {
	return {
		download: await get<Download>(fetch, '/api/download'),
		// The address the app connects to: this install, as the browser reached it.
		server: url.origin
	};
}
