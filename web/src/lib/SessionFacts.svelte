<!--
	@component
	What a session's flights add up to: who flew, and when, how long and how
	far, as two lists of facts beside the map. For a session and for a
	suggested one, which should look alike.
-->
<script lang="ts" module>
	import { areaShares, clock, dayMonth, distance, duration } from '#lib/format.js';

	interface Flight {
		takeoff_at: string;
		landed_at: string;
		air_time_s: number;
		distance_m: number;
		pilot_name: string | null;
		group_name: string | null;
		areas: { name: string; air_time_s: number }[];
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
			/** The flights' groups: those a suggested session would get. */
			groups: names(
				flights.map((flight) => flight.group_name),
				'None (organisation queue)'
			)
		};
	}

	/** A session's own group, as its facts say it. */
	export const sessionGroup = (session: { group_name: string | null }) =>
		session.group_name ?? 'None (organisation)';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		flights: Flight[];
		timeZone: string;
		/** A session's own group (see `sessionGroup`); without it, the
		 * flights' groups, as for a suggested session. */
		group?: string;
		/** E.g. a button to change the pilot, after the pilot's name. */
		pilotAction?: Snippet;
		/** E.g. a button to change the session's group, after its name. */
		groupAction?: Snippet;
	}

	let { flights, timeZone, group, pilotAction, groupAction }: Props = $props();

	const facts = $derived(summary(flights, timeZone));
	const distanceM = $derived(flights.reduce((total, flight) => total + flight.distance_m, 0));

	// Times only: the date is in the title (user's decision, 2026-10-21); a
	// landing after midnight says "+1".
	const nextDay = $derived(dayMonth(facts.end, timeZone) !== dayMonth(facts.start, timeZone));

	// In pairs, two columns: takeoff and landing, duration and air time
	// side by side, distance and flights; the area, if any, last and wide
	// (carbon-expert's grouping).
	const times = $derived([
		{ label: 'First takeoff', value: clock(facts.start, timeZone), numeric: true },
		{
			label: 'Last landing',
			value: `${clock(facts.end, timeZone)}${nextDay ? ' +1' : ''}`,
			numeric: true
		},
		{
			label: 'Duration',
			value: duration((Date.parse(facts.end) - Date.parse(facts.start)) / 1000),
			numeric: true
		},
		{ label: 'Air time', value: duration(facts.airTime), numeric: true },
		{ label: 'Distance', value: distance(distanceM), numeric: true },
		{ label: 'Flights', value: String(flights.length), numeric: true },
		// Only when the flights crossed an area.
		...[
			areaShares(
				flights.flatMap((flight) => flight.areas),
				facts.airTime
			)
		]
			.filter((fact) => fact !== null)
			.map((fact) => ({ ...fact, wide: true }))
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
		<dt>Group</dt>
		<dd class="with-action">
			{group ?? facts.groups}{#if groupAction}{@render groupAction()}{/if}
		</dd>
	</div>
</dl>
<dl class="facts pairs">
	{#each times as fact (fact.label)}
		<div class:wide={'wide' in fact}>
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

	.pairs {
		grid-template-columns: 1fr 1fr;
	}

	.wide {
		grid-column: 1 / -1;
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
