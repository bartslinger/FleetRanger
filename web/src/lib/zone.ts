// Days in the organisation's time zone, as filters use them: a day is from
// its midnight there to the next, not a UTC day, so evening flights count on
// the date people know them by.

/** Local time minus UTC, in milliseconds, of a time zone at an instant. */
function offset(at: number, timeZone: string): number {
	const parts = new Intl.DateTimeFormat('en-GB', {
		timeZone,
		year: 'numeric',
		month: 'numeric',
		day: 'numeric',
		hour: 'numeric',
		minute: 'numeric',
		second: 'numeric',
		hourCycle: 'h23'
	}).formatToParts(new Date(at));
	const value = (type: string) => Number(parts.find((part) => part.type === type)?.value);
	const local = Date.UTC(
		value('year'),
		value('month') - 1,
		value('day'),
		value('hour'),
		value('minute'),
		value('second')
	);
	return local - Math.floor(at / 1000) * 1000;
}

/** The instant a day ("2026-10-12") starts in a time zone. */
export function startOfDay(day: string, timeZone: string): Date {
	const [year, month, date] = day.split('-').map(Number);
	const midnight = Date.UTC(year, month - 1, date);
	// Twice: the offset at the first guess may differ around a daylight saving change.
	const guess = midnight - offset(midnight, timeZone);
	return new Date(midnight - offset(guess, timeZone));
}

/** The day after ("2026-10-13" for "2026-10-12"). */
export function nextDay(day: string): string {
	const [year, month, date] = day.split('-').map(Number);
	return new Date(Date.UTC(year, month - 1, date + 1)).toISOString().slice(0, 10);
}

/** A day as the URL and date fields write it: "2026-10-12", or null. */
export function isDay(value: string | null): value is string {
	return value !== null && /^\d{4}-\d{2}-\d{2}$/.test(value) && !Number.isNaN(Date.parse(value));
}
