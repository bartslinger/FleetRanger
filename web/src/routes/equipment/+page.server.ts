import { get, type Equipment } from '#lib/server/backend.js';
import { KINDS, parseKind } from '#lib/equipment.js';

export async function load({ fetch, url }) {
	// Both lists at once: they are short, and switching tabs needs no request.
	const [drones, controllers] = await Promise.all([
		get<Equipment[]>(fetch, KINDS.drones.api),
		get<Equipment[]>(fetch, KINDS.controllers.api)
	]);
	return { drones, controllers, tab: parseKind(url.searchParams.get('tab')) };
}
