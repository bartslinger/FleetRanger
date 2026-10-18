// Formatting for display. Times are shown in the organisation's time zone, so
// the server and the browser render the same text.

export function dateTime(iso: string, timeZone: string): string {
	return new Intl.DateTimeFormat('en-GB', {
		timeZone,
		day: 'numeric',
		month: 'short',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(new Date(iso));
}

/** Hours, minutes and seconds, e.g. 1:05:09. */
export function duration(seconds: number): string {
	const h = Math.floor(seconds / 3600);
	const m = Math.floor((seconds % 3600) / 60);
	const s = Math.floor(seconds % 60);
	return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** Kilometres with one decimal. */
export function distance(metres: number): string {
	return `${(metres / 1000).toFixed(1)} km`;
}
