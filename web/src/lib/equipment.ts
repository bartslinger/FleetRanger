// The two kinds of equipment, as tabs and in paths (/equipment/drones/…).

export type Kind = 'drones' | 'controllers';

export const KINDS: Record<Kind, { label: string; one: string; api: string }> = {
	drones: { label: 'Drones', one: 'Drone', api: '/api/aircraft' },
	controllers: {
		label: 'Remote controllers',
		one: 'Remote controller',
		api: '/api/remote-controllers'
	}
};

export function parseKind(value: string | null): Kind {
	return value === 'controllers' ? 'controllers' : 'drones';
}
