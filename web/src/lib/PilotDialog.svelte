<!--
	@component
	The dialog that sets the pilot of some flights, posting `pilot_id` and a
	`flight_id` per flight to the page's `?/setPilot`: all of a session's, the
	selected ones, or one. Opened by the page, e.g. from a pencil or the
	batch bar; the same rules as the flight page's.
-->
<script lang="ts">
	import { pilotOptions, pilotValue } from '#lib/pilots.js';
	import Modal from '#lib/ui/Modal.svelte';
	import Select from '#lib/ui/Select.svelte';

	interface Props {
		/** The flights to change. */
		flights: { id: string; pilot_id: string | null; team_id: string | null }[];
		/** Which flights those are, e.g. "All 3 flights in the session." */
		which: string;
		choices: Parameters<typeof pilotOptions>[0];
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

	const options = $derived(pilotOptions(choices));
	// Their pilot, with their team, if they share them.
	const choice = $derived.by(() => {
		const values = new Set(flights.map((flight) => pilotValue(flight.pilot_id, flight.team_id)));
		const [shared] = values;
		return values.size === 1 && options.some((option) => option.value === shared) ? shared : '';
	});
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
	<Select label="Pilot" name="pilot" {options} value={choice} />
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
