// The two kinds of equipment with a team, as tabs and in paths
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
