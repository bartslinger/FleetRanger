<script lang="ts">
	import { resolve } from '$app/paths';
	import { dateTime, daySpan, distance, duration } from '#lib/format.js';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const flights = $derived(data.session.flights);
	type Row = (typeof flights)[number];

	/** Distinct names in order of first appearance. */
	const names = (values: (string | null)[], none: string) =>
		[...new Set(values.map((value) => value ?? none))].join(', ');

	const start = $derived(flights[0].takeoff_at);
	const end = $derived(
		flights.reduce((last, flight) => (flight.landed_at > last ? flight.landed_at : last), start)
	);
	const sum = (value: (flight: Row) => number) =>
		flights.reduce((total, flight) => total + value(flight), 0);

	const groups = $derived([
		[
			{
				label: 'Pilot',
				value: names(
					flights.map((f) => f.pilot_name),
					'Unknown'
				)
			},
			{
				label: 'Team',
				value: names(
					flights.map((f) => f.team_name),
					'None (organisation queue)'
				)
			}
		],
		[
			{ label: `First takeoff (${data.timeZoneName})`, value: dateTime(start, data.timeZone) },
			{ label: `Last landing (${data.timeZoneName})`, value: dateTime(end, data.timeZone) },
			{
				label: 'Duration',
				value: duration((Date.parse(end) - Date.parse(start)) / 1000),
				numeric: true
			},
			{ label: 'Air time', value: duration(sum((f) => f.air_time_s)), numeric: true },
			{ label: 'Distance', value: distance(sum((f) => f.distance_m)), numeric: true },
			{ label: 'Flights', value: String(flights.length), numeric: true }
		]
	]);

	// Splitting: admins change any session, members those of their own team.
	let selected: string[] = $state([]);
	const canChange = $derived(
		data.choices.admin ||
			(data.choices.own_team_id !== null &&
				flights.every((flight) => flight.team_id === data.choices.own_team_id))
	);

	function moved(count: number) {
		selected = [];
		toast(`${count} ${count === 1 ? 'flight' : 'flights'} moved to a new session.`);
	}

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: `Takeoff (${data.timeZoneName})`,
			format: (flight) => dateTime(flight.takeoff_at, data.timeZone)
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (flight) => duration(flight.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			format: (flight) => distance(flight.distance_m)
		},
		{ key: 'pilot_name', header: 'Pilot' },
		{ key: 'aircraft_serial', header: 'Drone serial', identifier: true }
	]);
</script>

<PageHeader
	title={daySpan(start, end, data.timeZone, true)}
	breadcrumb={[{ href: resolve('/sessions'), label: 'Sessions' }]}
/>

<Grid rowGap>
	<Column narrow lg={11}>
		<TrackMap
			tracks={flights.map((flight) => flight.track)}
			token={data.mapboxToken}
			label="Map of the session’s tracks"
		/>
	</Column>
	<Column lg={5}>
		{#each groups as facts, index (index)}
			<dl class="facts">
				{#each facts as fact (fact.label)}
					<div>
						<dt>{fact.label}</dt>
						<dd class:numeric={fact.numeric}>{fact.value}</dd>
					</div>
				{/each}
			</dl>
		{/each}
	</Column>
	<Column narrow>
		<DataTable
			label="Flights of the session"
			{columns}
			rows={flights}
			rowKey={(flight) => flight.id}
			rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
			selectable={canChange ? () => true : undefined}
			bind:selected
			selectedLabel={(count) => `${count} ${count === 1 ? 'flight' : 'flights'} selected`}
		>
			{#snippet batchActions(ids)}
				{#if ids.length < flights.length}
					<Modal
						title="Move to new session"
						submitLabel="Move"
						action="?/split"
						error={form?.splitMessage}
						onsuccess={() => moved(ids.length)}
					>
						{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
						<p>
							The {ids.length === 1 ? 'flight becomes' : `${ids.length} flights become`} a session of
							{ids.length === 1 ? 'its' : 'their'} own. Merge sessions in the list to undo this.
						</p>
					</Modal>
				{/if}
			{/snippet}
		</DataTable>
	</Column>
</Grid>

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

	.numeric {
		font-variant-numeric: tabular-nums slashed-zero;
	}
</style>
