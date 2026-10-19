<!--
	@component
	The dialog that sets the pilot of some flights, posting `pilot_id` and a
	`flight_id` per flight to the page's `?/setPilot`: all of a session's, the
	selected ones, or one. Opened by the page, e.g. from a pencil or the
	batch bar; the same rules as the flight page's.
-->
<script lang="ts">
	import Modal from '#lib/ui/Modal.svelte';
	import Select from '#lib/ui/Select.svelte';

	interface Props {
		/** The flights to change. */
		flights: { id: string; pilot_id: string | null }[];
		/** Which flights those are, e.g. "All 3 flights in the session." */
		which: string;
		choices: {
			admin: boolean;
			pilots: { id: string; name: string; team_name: string }[];
		};
		open: boolean;
		returnFocus?: () => HTMLElement | undefined;
		error?: string;
		onsuccess?: (data: Record<string, unknown> | undefined) => void;
	}

	let {
		flights,
		which,
		choices,
		open = $bindable(),
		returnFocus,
		error,
		onsuccess
	}: Props = $props();

	const options = $derived([
		{ value: '', label: 'Unknown' },
		...choices.pilots.map((pilot) => ({
			value: pilot.id,
			label: choices.admin ? `${pilot.name} (${pilot.team_name})` : pilot.name
		}))
	]);
	// Their pilot if they share one.
	const choice = $derived(
		new Set(flights.map((flight) => flight.pilot_id)).size === 1 ? (flights[0]?.pilot_id ?? '') : ''
	);
</script>

<Modal
	title="Change pilot"
	submitLabel="Save"
	action="?/setPilot"
	{error}
	{onsuccess}
	trigger={false}
	bind:open
	{returnFocus}
>
	{#each flights as flight (flight.id)}
		<input type="hidden" name="flight_id" value={flight.id} />
	{/each}
	<Select label="Pilot" name="pilot_id" {options} value={choice} />
	<p class="help">
		{which}
		{choices.admin
			? 'A pilot of another team moves the flights to that team. With Unknown, they keep theirs.'
			: 'Choose Unknown for flights not flown by someone in your team.'}
	</p>
</Modal>

<style>
	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
