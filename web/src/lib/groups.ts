// Choosing a group for flights, sessions or equipment: only those where the
// user holds the permission, and the organisation itself (no group, value
// "") when they hold it there.

import { may, type Permission } from '#lib/permissions.js';
import type { Account } from '#lib/server/backend.js';

/** A group's id and name, as flights and sessions carry them. */
interface Grouped {
	group_id: string | null;
	group_name: string | null;
}

/** The group these share, or null when they have several or none. */
export function sharedGroup(items: Grouped[]): { id: string; name: string } | null {
	const [first] = items;
	if (!first?.group_id || items.some((item) => item.group_id !== first.group_id)) return null;
	return { id: first.group_id, name: first.group_name ?? '' };
}

/**
 * The choice of a new session's group: where the user may edit sessions,
 * the organisation ("organisation") last, and the flights' shared group
 * chosen if it is one of them, else nothing (user's decisions on the
 * session's group; wording and preselection the carbon-expert's advice).
 * With one option the form posts it without showing a field.
 */
export function sessionGroupChoice(
	account: Account | null | undefined,
	groups: { id: string; name: string }[],
	flights: Grouped[]
): { options: { value: string; label: string }[]; value: string } {
	const options = groupOptions(account, 'sessions.edit', groups).map((option) =>
		option.value === '' ? { ...option, value: 'organisation' } : option
	);
	// Flights all in no group (the queue) share the organisation.
	const shared =
		flights.length > 0 && flights.every((flight) => flight.group_id === null)
			? 'organisation'
			: sharedGroup(flights)?.id;
	const value =
		options.length === 1
			? options[0].value
			: (options.find((option) => option.value === shared)?.value ?? '');
	return { options, value };
}

/** The group a new session gets, as SessionGroupField posts it: an id, or
 * "organisation" for none; null if nothing was chosen. Without the field,
 * the backend gives the flights' group when they share one. */
export function sessionGroup(form: FormData): { group_id?: string | null } | null {
	const group = form.get('group_id');
	if (group === null) return {};
	if (group === '') return null;
	return { group_id: group === 'organisation' ? null : String(group) };
}

/** What sits in no group is in the organisation, shown as the top group
 * (user's decision): one word everywhere. */
export const ORGANISATION = 'Organisation';

/**
 * The organisation first, as the top group, then the groups to choose, by name.
 *
 * - `groups`: the groups the user may choose from (`/api/flights/choices`).
 */
export function groupOptions(
	account: Account | null | undefined,
	permission: Permission,
	groups: { id: string; name: string }[]
): { value: string; label: string }[] {
	return [
		...(may(account, permission, null) ? [{ value: '', label: ORGANISATION }] : []),
		...groups
			.filter((group) => may(account, permission, group.id))
			.map((group) => ({ value: group.id, label: group.name }))
	];
}
