const RAD = Math.PI / 180;
const DAY_MS = 86400000;
const J1970 = 2440588;
const J2000 = 2451545;
const J0 = 0.0009;

/**
 * Sunrise and sunset (the sun's upper edge on the horizon, 0.833° below it
 * with refraction) on the day of an instant, at a place: SunCalc's formulas,
 * to about a minute. Null in polar day or night.
 */
export function sunriseSunset(at: string, lon: number, lat: number): [Date, Date] | null {
	const days = Date.parse(at) / DAY_MS - 0.5 + J1970 - J2000;
	const lw = -lon * RAD;
	const phi = lat * RAD;
	// The solar noon nearest the instant.
	const n = Math.round(days - J0 - lw / (2 * Math.PI));
	const ds = J0 + lw / (2 * Math.PI) + n;
	const M = (357.5291 + 0.98560028 * ds) * RAD;
	const C = (1.9148 * Math.sin(M) + 0.02 * Math.sin(2 * M) + 0.0003 * Math.sin(3 * M)) * RAD;
	const L = M + C + 102.9372 * RAD + Math.PI;
	const declination = Math.asin(Math.sin(L) * Math.sin(23.4397 * RAD));
	const noon = J2000 + ds + 0.0053 * Math.sin(M) - 0.0069 * Math.sin(2 * L);
	const hourAngle = Math.acos(
		(Math.sin(-0.833 * RAD) - Math.sin(phi) * Math.sin(declination)) /
			(Math.cos(phi) * Math.cos(declination))
	);
	if (Number.isNaN(hourAngle)) return null;
	const date = (julian: number) => new Date((julian + 0.5 - J1970) * DAY_MS);
	return [date(noon - hourAngle / (2 * Math.PI)), date(noon + hourAngle / (2 * Math.PI))];
}
