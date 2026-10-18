// Formatting for display. Times are shown in the organisation's time zone.
//
// The server and the browser must produce identical text, or the page changes
// after loading. Their locale data differs ("12 Oct 2026, 17:49" in Node,
// "12 Oct 2026 at 17:49" in a browser, "Sept" or "Sep"), so only numbers are
// taken from Intl and the text is assembled here.

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** The date and time parts of an instant in a time zone. */
function parts(iso: string, timeZone: string) {
	const values = new Intl.DateTimeFormat('en-GB', {
		timeZone,
		year: 'numeric',
		month: 'numeric',
		day: 'numeric',
		hour: 'numeric',
		minute: 'numeric',
		hourCycle: 'h23'
	})
		.formatToParts(new Date(iso))
		.filter((part) => part.type !== 'literal');
	const value = (type: string) => Number(values.find((part) => part.type === type)?.value);
	return {
		year: value('year'),
		month: value('month'),
		day: value('day'),
		hour: value('hour'),
		minute: value('minute')
	};
}

const pad = (value: number) => String(value).padStart(2, '0');

/** For example "12 Oct 2026, 17:49". */
export function dateTime(iso: string, timeZone: string): string {
	const { year, month, day, hour, minute } = parts(iso, timeZone);
	return `${day} ${MONTHS[month - 1]} ${year}, ${pad(hour)}:${pad(minute)}`;
}

/** The time zone's abbreviation, e.g. "UTC" or "GMT+2", as it is now. */
export function timeZoneName(timeZone: string): string {
	return (
		new Intl.DateTimeFormat('en-GB', { timeZone, timeZoneName: 'short' })
			.formatToParts(new Date())
			.find((part) => part.type === 'timeZoneName')?.value ?? timeZone
	);
}

/** Hours, minutes and seconds, e.g. 1:05:09. */
export function duration(seconds: number): string {
	const h = Math.floor(seconds / 3600);
	const m = Math.floor((seconds % 3600) / 60);
	const s = Math.floor(seconds % 60);
	return `${h}:${pad(m)}:${pad(s)}`;
}

/** Kilometres with one decimal; "<0.1 km" rather than a misleading "0.0 km". */
export function distance(metres: number): string {
	if (metres < 50) return '<0.1 km';
	return `${(metres / 1000).toFixed(1)} km`;
}
