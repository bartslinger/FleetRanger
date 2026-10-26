// An app's return addresses, entered one per line on the Apps pages. The
// backend checks them too; this explains a mistake before sending.

import { dev } from '$app/env';

/** The non-empty lines of the text area. */
export const addressLines = (text: string) =>
	text
		.split('\n')
		.map((line) => line.trim())
		.filter((line) => line !== '');

/** HTTPS only, as the backend asks of trusted apps; the dev server, run
 * against a debug backend, also allows http:// to this computer, so an app
 * developed locally can be trusted. */
function valid(address: string): boolean {
	try {
		const url = new URL(address);
		const loopback = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname);
		return (
			!address.includes('#') &&
			(url.protocol === 'https:' || (dev && url.protocol === 'http:' && loopback))
		);
	} catch {
		return false;
	}
}

/** What's wrong with the addresses, naming the line (carbon-expert's advice:
 * one field holds them all), or undefined if they're fine. */
export function addressError(text: string): string | undefined {
	const lines = text.split('\n').map((line) => line.trim());
	const filled = lines.filter((line) => line !== '');
	if (filled.length === 0) return 'Enter at least one return address.';
	if (filled.length > 10) return 'Enter no more than 10 return addresses.';
	const bad = lines.flatMap((line, index) => (line !== '' && !valid(line) ? [index + 1] : []));
	if (bad.length > 0) {
		const others = bad.length > 1 ? ` ${bad.length - 1} more lines aren’t either.` : '';
		const start = dev ? 'https://, or http:// on this computer' : 'https://';
		return `Line ${bad[0]} isn’t a valid web address. Use the full address, starting with ${start}.${others}`;
	}
	const seen = new Map<string, number>();
	for (const [index, line] of lines.entries()) {
		if (line === '') continue;
		const first = seen.get(line);
		if (first !== undefined) return `Line ${index + 1} repeats line ${first}.`;
		seen.set(line, index + 1);
	}
	return undefined;
}
