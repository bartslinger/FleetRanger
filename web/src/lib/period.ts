// Weeks and months of days ("2026-10-12"), as the Sessions cards page steps
// through them. Weeks start on Monday (user's decision, 2026-10-21).

import { dayMonth, month as monthName } from '#lib/format.js';

export interface Period {
	unit: 'week' | 'month';
	/** The first day: a Monday, or the 1st. */
	start: string;
}

const utc = (day: string) => {
	const [year, month, date] = day.split('-').map(Number);
	return new Date(Date.UTC(year, month - 1, date));
};
const text = (date: Date) => date.toISOString().slice(0, 10);

/** Today in a time zone. */
export function today(timeZone: string): string {
	return dayOf(new Date().toISOString(), timeZone);
}

/** The day of an instant in a time zone, e.g. "2026-10-21". */
export function dayOf(iso: string, timeZone: string): string {
	// en-CA writes dates as 2026-10-21.
	return new Intl.DateTimeFormat('en-CA', { timeZone }).format(new Date(iso));
}

/** The period of a unit that a day is in. */
export function periodOf(unit: Period['unit'], day: string): Period {
	const date = utc(day);
	if (unit === 'month') return { unit, start: `${day.slice(0, 7)}-01` };
	// getUTCDay: Sunday is 0; back to Monday.
	date.setUTCDate(date.getUTCDate() - ((date.getUTCDay() + 6) % 7));
	return { unit, start: text(date) };
}

/**
 * The same moment in another unit (user's request, 2026-10-21): a week's
 * month is the one its Monday is in; a month's week is the current one if
 * today is in that month, else the one with its 1st.
 */
export function switchTo(unit: Period['unit'], period: Period, today: string): Period {
	if (unit === period.unit) return period;
	if (unit === 'month') return periodOf('month', period.start);
	const { from, to } = days(period);
	return periodOf('week', today >= from && today <= to ? today : period.start);
}

/** The period before (-1) or after (1). */
export function step(period: Period, by: -1 | 1): Period {
	const date = utc(period.start);
	if (period.unit === 'month') date.setUTCMonth(date.getUTCMonth() + by);
	else date.setUTCDate(date.getUTCDate() + 7 * by);
	return { unit: period.unit, start: text(date) };
}

/** Its first and last day. */
export function days(period: Period): { from: string; to: string } {
	const last = utc(step(period, 1).start);
	last.setUTCDate(last.getUTCDate() - 1);
	return { from: period.start, to: text(last) };
}

/** "October 2026", or a week: "12–18 Oct 2026", "28 Sep–4 Oct 2026". */
export function periodName(period: Period): string {
	const at = (day: string) => utc(day).toISOString();
	if (period.unit === 'month') return monthName(at(period.start), 'UTC');
	const { from, to } = days(period);
	const year = to.slice(0, 4);
	if (from.slice(0, 4) !== year) {
		return `${dayMonth(at(from), 'UTC')} ${from.slice(0, 4)}–${dayMonth(at(to), 'UTC')} ${year}`;
	}
	if (from.slice(0, 7) === to.slice(0, 7)) {
		return `${Number(from.slice(8))}–${dayMonth(at(to), 'UTC')} ${year}`;
	}
	return `${dayMonth(at(from), 'UTC')}–${dayMonth(at(to), 'UTC')} ${year}`;
}
