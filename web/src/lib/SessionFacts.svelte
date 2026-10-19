<!--
	@component
	What a session's flights add up to: who flew, and when, how long and how
	far, as two lists of facts beside the map. For a session and for a
	suggested one, which should look alike.
-->
<script lang="ts" module>
	import { clock, dateTime, distance, duration } from '#lib/format.js';

	interface Flight {
		takeoff_at: string;
		landed_at: string;
		air_time_s: number;
		distance_m: number;
		pilot_name: string | null;
		team_name: string | null;
	}

	/** Distinct names in order of first appearance. */
	const names = (values: (string | null)[], none: string) =>
		[...new Set(values.map((value) => value ?? none))].join(', ');

	/** The span and the names of a session's flights, also for its page header. */
	export function summary(flights: Flight[], timeZone: string) {
		const start = flights[0].takeoff_at;
		const end = flights.reduce(
			(last, flight) => (flight.landed_at > last ? flight.landed_at : last),
			start
		);
		return {
			start,
			end,
			span: `${clock(start, timeZone)}–${clock(end, timeZone)}`,
			airTime: flights.reduce((total, flight) => total + flight.air_time_s, 0),
			pilots: names(
				flights.map((flight) => flight.pilot_name),
				'Unknown'
			),
			teams: names(
				flights.map((flight) => flight.team_name),
				'None (organisation queue)'
			)
		};
	}
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		flights: Flight[];
		timeZone: string;
		timeZoneName: string;
		/** E.g. a button to change the pilot, after the pilot's name. */
		pilotAction?: Snippet;
	}

	let { flights, timeZone, timeZoneName, pilotAction }: Props = $props();

	const facts = $derived(summary(flights, timeZone));
	const distanceM = $derived(flights.reduce((total, flight) => total + flight.distance_m, 0));

	const times = $derived([
		{ label: `First takeoff (${timeZoneName})`, value: dateTime(facts.start, timeZone) },
		{ label: `Last landing (${timeZoneName})`, value: dateTime(facts.end, timeZone) },
		{
			label: 'Duration',
			value: duration((Date.parse(facts.end) - Date.parse(facts.start)) / 1000),
			numeric: true
		},
		{ label: 'Air time', value: duration(facts.airTime), numeric: true },
		{ label: 'Distance', value: distance(distanceM), numeric: true },
		{ label: 'Flights', value: String(flights.length), numeric: true }
	]);
</script>

<dl class="facts">
	<div>
		<dt>Pilot</dt>
		<dd class="with-action">
			{facts.pilots}{#if pilotAction}{@render pilotAction()}{/if}
		</dd>
	</div>
	<div>
		<dt>Team</dt>
		<dd>{facts.teams}</dd>
	</div>
</dl>
<dl class="facts">
	{#each times as fact (fact.label)}
		<div>
			<dt>{fact.label}</dt>
			<dd class:numeric={fact.numeric}>{fact.value}</dd>
		</div>
	{/each}
</dl>

<style>
	/* Related facts 16px apart; 32px between groups. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-07);
	}

	dt {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.with-action {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
	}

	.numeric {
		font-variant-numeric: tabular-nums slashed-zero;
	}
</style>
