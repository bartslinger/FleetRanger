// The two kinds of equipment with a group, as tabs and in paths
// (/equipment/drones/…). Batteries have a tab and pages of their own.

export type Kind = 'drones' | 'controllers';

/** The Equipment page's tabs. */
export type Tab = Kind | 'batteries';

export const KINDS: Record<Kind, { label: string; one: string; api: string }> = {
	drones: { label: 'Drones', one: 'Drone', api: '/api/aircraft' },
	controllers: {
		label: 'Remote controllers',
		one: 'Remote controller',
		api: '/api/remote-controllers'
	}
};

export function parseTab(value: string | null): Tab {
	return value === 'controllers' || value === 'batteries' ? value : 'drones';
}

/** Who flew a device last: their name, or for a flight the user doesn't see,
 * only that someone did (carbon-expert's wording; Carbon has no rule for a
 * withheld value). */
export function lastPilot(item: {
	last_flight_at: string | null;
	last_pilot_name: string | null;
	last_pilot_hidden: boolean;
}): string | null {
	if (!item.last_flight_at) return null;
	if (item.last_pilot_hidden) return 'Another pilot';
	return item.last_pilot_name ?? 'Unknown pilot';
}
