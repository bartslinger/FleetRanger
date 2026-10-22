import { get, type Battery, type Equipment } from '#lib/server/backend.js';
import { KINDS, parseTab } from '#lib/equipment.js';

export async function load({ fetch, url }) {
	// All lists at once: they are short, and switching tabs needs no request.
	const [drones, controllers, batteries] = await Promise.all([
		get<Equipment[]>(fetch, KINDS.drones.api),
		get<Equipment[]>(fetch, KINDS.controllers.api),
		get<Battery[]>(fetch, '/api/batteries')
	]);
	return { drones, controllers, batteries, tab: parseTab(url.searchParams.get('tab')) };
}
