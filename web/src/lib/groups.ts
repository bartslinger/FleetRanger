// Choosing a group for flights, sessions or equipment: only those where the
// user holds the permission. Everything is in a group; while the
// organisation has only one, no group is shown or chosen (user's decision,
// 2026-11-11).

import { may, type Permission } from '#lib/permissions.js';
import type { Account } from '#lib/server/backend.js';

/** A group's id and name, as flights and sessions carry them. */
interface Grouped {
	group_id: string;
	group_name: string;
}

/** Whether groups are shown at all: once the organisation has several. */
export function showsGroups(account: Account | null | undefined): boolean {
	return account?.several_groups ?? false;
}

/** The group these share, or null when they have several. */
export function sharedGroup(items: Grouped[]): { id: string; name: string } | null {
	const [first] = items;
	if (!first || items.some((item) => item.group_id !== first.group_id)) return null;
	return { id: first.group_id, name: first.group_name };
}

/**
 * The choice of a new session's group: where the user may edit sessions,
 * and the flights' shared group chosen if it is one of them, else nothing
 * (user's decisions on the session's group; wording and preselection the
 * carbon-expert's advice). With one option the form posts it without
 * showing a field.
 */
export function sessionGroupChoice(
	account: Account | null | undefined,
	groups: { id: string; name: string }[],
	flights: Grouped[]
): { options: { value: string; label: string }[]; value: string } {
	const options = groupOptions(account, 'sessions.edit', groups);
	const shared = sharedGroup(flights)?.id;
	const value =
		options.length === 1
			? options[0].value
			: (options.find((option) => option.value === shared)?.value ?? '');
	return { options, value };
}

/** The group a new session gets, as SessionGroupField posts it; null if
 * nothing was chosen. Without the field, the backend gives the flights'
 * group when they share one. */
export function sessionGroup(form: FormData): { group_id?: string } | null {
	const group = form.get('group_id');
	if (group === null) return {};
	if (group === '') return null;
	return { group_id: String(group) };
}

/**
 * The groups to choose, by name.
 *
 * - `groups`: the groups the user may choose from (`/api/flights/choices`).
 */
export function groupOptions(
	account: Account | null | undefined,
	permission: Permission,
	groups: { id: string; name: string }[]
): { value: string; label: string }[] {
	return groups
		.filter((group) => may(account, permission, group.id))
		.map((group) => ({ value: group.id, label: group.name }));
}
