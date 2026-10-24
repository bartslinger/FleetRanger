// Choosing a pilot: everyone who flies, and the flights stay in their group.
// A choice's value is the pilot's id; empty for Unknown.

import type { Pilot } from '#lib/server/backend.js';

interface Choices {
	pilots: Pilot[];
}

type Option = { value: string; label: string };

/**
 * The options of a pilot field: Unknown first, then, for flights of one
 * group, that group's own pilots under its name and the others after them
 * (the carbon-expert's suggestion; Carbon is silent on option groups);
 * otherwise one list.
 */
export function pilotOptions(
	choices: Choices,
	group: { id: string; name: string } | null = null
): (Option | { group: string; options: Option[] })[] {
	const unknown = { value: '', label: 'Unknown' };
	const option = (pilot: Pilot) => ({ value: pilot.id, label: pilot.name });
	const own = group ? choices.pilots.filter((pilot) => pilot.group_ids.includes(group.id)) : [];
	if (!group || own.length === 0) return [unknown, ...choices.pilots.map(option)];
	const others = choices.pilots.filter((pilot) => !pilot.group_ids.includes(group.id));
	return [
		unknown,
		{ group: `${group.name} pilots`, options: own.map(option) },
		...(others.length > 0 ? [{ group: 'Other pilots', options: others.map(option) }] : [])
	];
}

/** Whether a pilot can be chosen, e.g. to preselect them. */
export function offered(choices: Choices, value: string): boolean {
	return value === '' || choices.pilots.some((pilot) => pilot.id === value);
}

/** A chosen value as the backend takes it. */
export function parsePilot(value: string): { pilot_id: string | null } {
	return { pilot_id: value || null };
}
