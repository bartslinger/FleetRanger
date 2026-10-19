// Choosing a pilot: a pilot in several teams is a choice per team, and the
// flights move to the team chosen (user's decision, 2026-10-19). A choice's
// value is "pilot id:team id"; empty for Unknown.

interface Choices {
	admin: boolean;
	own_team_ids: string[];
	pilots: { id: string; name: string; team_id: string; team_name: string }[];
}

/** The options of a pilot field, Unknown first. The team shows for admins and
 * members of several teams, who choose between teams. */
export function pilotOptions(choices: Choices): { value: string; label: string }[] {
	const withTeam = choices.admin || choices.own_team_ids.length > 1;
	return [
		{ value: '', label: 'Unknown' },
		...choices.pilots.map((pilot) => ({
			value: pilotValue(pilot.id, pilot.team_id),
			label: withTeam ? `${pilot.name} (${pilot.team_name})` : pilot.name
		}))
	];
}

/** The value for a flight's pilot with its team; empty without a pilot. */
export function pilotValue(pilotId: string | null, teamId: string | null): string {
	return pilotId ? `${pilotId}:${teamId ?? ''}` : '';
}

/** A chosen value as the backend takes it. */
export function parsePilot(value: string): { pilot_id: string | null; team_id: string | null } {
	const [pilot, team] = value.split(':');
	return { pilot_id: pilot || null, team_id: team || null };
}
