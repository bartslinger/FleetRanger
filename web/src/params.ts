import { defineParams } from '@sveltejs/kit/params';
import type { Kind } from '#lib/equipment.js';

export const params = defineParams({
	/** The kinds of equipment in paths such as /equipment/drones/…. */
	kind: (param): Kind | undefined =>
		param === 'drones' || param === 'controllers' ? param : undefined
});
