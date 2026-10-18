import { defineEnvVars } from '@sveltejs/kit/env';

export const variables = defineEnvVars({
	BACKEND_URL: {
		description: 'Where the server reaches the backend.',
		schema: (value) => new URL(value || 'http://127.0.0.1:8080').href.replace(/\/$/, '')
	},
	MAPBOX_ACCESS_TOKEN: {
		description: "The install's public Mapbox token (pk.…); without one, pages show no map.",
		schema: (value) => {
			if (value && !value.startsWith('pk.')) {
				throw new Error('Use a public Mapbox token (pk.…): the browser sees it.');
			}
			return value ?? '';
		}
	},
	TIME_ZONE: {
		description: "The organisation's time zone; all times are shown in it.",
		schema: (value) => {
			const zone = value || 'UTC';
			// Throws a RangeError for unknown zones.
			new Intl.DateTimeFormat('en', { timeZone: zone });
			return zone;
		}
	}
});
