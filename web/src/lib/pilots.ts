// Choosing a pilot: everyone who flies, and the flights stay in their group.
// A choice's value is the pilot's id; empty for Unknown.

interface Choices {
	pilots: { id: string; name: string }[];
}

/** The options of a pilot field, Unknown first. */
export function pilotOptions(choices: Choices): { value: string; label: string }[] {
	return [
		{ value: '', label: 'Unknown' },
		...choices.pilots.map((pilot) => ({ value: pilot.id, label: pilot.name }))
	];
}

/** A chosen value as the backend takes it. */
export function parsePilot(value: string): { pilot_id: string | null } {
	return { pilot_id: value || null };
}
