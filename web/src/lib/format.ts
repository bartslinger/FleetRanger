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

const MONTH_NAMES = [
	'January',
	'February',
	'March',
	'April',
	'May',
	'June',
	'July',
	'August',
	'September',
	'October',
	'November',
	'December'
];
const WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

/** For example "October 2026": a heading over that month's items. */
export function month(iso: string, timeZone: string): string {
	const { year, month } = parts(iso, timeZone);
	return `${MONTH_NAMES[month - 1]} ${year}`;
}

/** For example "Mon 12 Oct", or "Mon 12 Oct 2026" with the year. */
export function day(iso: string, timeZone: string, withYear = false): string {
	const { year, month, day } = parts(iso, timeZone);
	const weekday = WEEKDAYS[new Date(Date.UTC(year, month - 1, day)).getUTCDay()];
	return `${weekday} ${day} ${MONTHS[month - 1]}${withYear ? ` ${year}` : ''}`;
}

/** For example "12 Oct": short, where the year is clear from the context. */
export function dayMonth(iso: string, timeZone: string): string {
	const { month, day } = parts(iso, timeZone);
	return `${day} ${MONTHS[month - 1]}`;
}

/** For example "18:32". */
export function clock(iso: string, timeZone: string): string {
	const { hour, minute } = parts(iso, timeZone);
	return `${pad(hour)}:${pad(minute)}`;
}

/**
 * The day and times of a span, for example "Mon 12 Oct, 18:32–21:01". Night
 * work can end after midnight; the end is then the next morning's time.
 */
export function daySpan(start: string, end: string, timeZone: string, withYear = false): string {
	return `${day(start, timeZone, withYear)}, ${clock(start, timeZone)}–${clock(end, timeZone)}`;
}

/** The time zone's abbreviation, e.g. "UTC" or "GMT+2", as it is now. */
export function timeZoneName(timeZone: string): string {
	return (
		new Intl.DateTimeFormat('en-GB', { timeZone, timeZoneName: 'short' })
			.formatToParts(new Date())
			.find((part) => part.type === 'timeZoneName')?.value ?? timeZone
	);
}

/**
 * With units, to the nearest minute: "2 h 19 min", "21 min"; under a minute
 * in seconds, "45 s". A bare 0:21:29 was easy to misread.
 */
export function duration(seconds: number): string {
	if (seconds < 59.5) return `${Math.round(seconds)} s`;
	const minutes = Math.round(seconds / 60);
	const h = Math.floor(minutes / 60);
	const m = minutes % 60;
	if (h === 0) return `${m} min`;
	return m === 0 ? `${h} h` : `${h} h ${m} min`;
}

/** A download's size, as people read it: "640 kB", "12.4 MB". */
export function fileSize(bytes: number): string {
	if (bytes < 1_000_000) return `${Math.ceil(bytes / 1000)} kB`;
	return `${(bytes / 1_000_000).toFixed(1)} MB`;
}

/** Kilometres with one decimal; "<0.1 km" rather than a misleading "0.0 km". */
export function distance(metres: number): string {
	if (metres < 50) return '<0.1 km';
	return `${(metres / 1000).toFixed(1)} km`;
}
