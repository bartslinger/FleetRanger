<script lang="ts">
	import { untrack } from 'svelte';
	import { resolve } from '$app/paths';
	import { day, distance, duration, timeSpan } from '#lib/format.js';
	import AreaFigures from '#lib/AreaFigures.svelte';
	import type { FlightArea } from '#lib/server/backend.js';
	import SessionFacts, { summary } from '#lib/SessionFacts.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Edit from '@carbon/icons/es/edit/16.js';
	import PilotDialog from '#lib/PilotDialog.svelte';
	import Button from '#lib/ui/Button.svelte';
	import IconButton from '#lib/ui/IconButton.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import SessionStepper from './SessionStepper.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const flights = $derived(data.session.flights);
	type Row = (typeof flights)[number];

	const facts = $derived(summary(flights, data.timeZone));

	// The session's time and distance per area: its flights' added up, most time first.
	const areas = $derived.by(() => {
		const totals: FlightArea[] = [];
		for (const area of flights.flatMap((flight) => flight.areas)) {
			const total = totals.find((other) => other.id === area.id);
			if (total) {
				total.air_time_s += area.air_time_s;
				total.distance_m += area.distance_m;
			} else {
				totals.push({ ...area });
			}
		}
		return totals.sort((a, b) => b.air_time_s - a.air_time_s || a.name.localeCompare(b.name));
	});

	// Splitting: admins change any session, members those of their own team.
	let selected: string[] = $state([]);
	// Stepping to another session starts without a selection.
	let selectedFor = untrack(() => data.session.id);
	$effect(() => {
		if (data.session.id === selectedFor) return;
		selectedFor = data.session.id;
		selected = [];
	});
	// The flight pointed at in the table, brought out on the map.
	let pointed: number | null = $state(null);
	const canChange = $derived(
		data.choices.admin ||
			flights.every(
				(flight) => flight.team_id !== null && data.choices.own_team_ids.includes(flight.team_id)
			)
	);

	function moved(count: number) {
		selected = [];
		toast(`${count} ${count === 1 ? 'flight' : 'flights'} moved to a new session.`);
	}

	function removed(count: number) {
		selected = [];
		toast(`${count} ${count === 1 ? 'flight' : 'flights'} removed from the session.`);
	}

	const flightsLabel = (count: number) => `${count} ${count === 1 ? 'flight' : 'flights'}`;

	// The pilot of all its flights or the selected ones, in one dialog.
	let pilotOpen = $state(false);
	let pilotFor: Row[] = $state([]);
	let pilotWhich = $state('');
	let pilotOpener: HTMLElement | undefined;
	function changePilot(of: Row[], which: string, event: Event) {
		pilotFor = of;
		pilotWhich = which;
		pilotOpener = event.currentTarget as HTMLElement;
		pilotOpen = true;
	}
	function pilotSet(result: Record<string, unknown> | undefined) {
		selected = [];
		const id = result?.pilotId as string | null;
		const name = data.choices.pilots.find((pilot) => pilot.id === id)?.name ?? 'Unknown';
		toast(`Pilot set to ${name} for ${flightsLabel(Number(result?.changed ?? 0))}.`);
	}

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: `Time (${data.timeZoneName})`,
			format: (flight) => timeSpan(flight.takeoff_at, flight.landed_at, data.timeZone)
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
	title={`Session ${day(facts.start, data.timeZone, true)}`}
	crumb={`Session ${day(facts.start, data.timeZone)}`}
	description={`${facts.span} · ${flights.length} ${flights.length === 1 ? 'flight' : 'flights'} · ${duration(facts.airTime)} in the air`}
	phoneDescription={`${facts.span} · ${facts.pilots} · ${facts.teams}`}
	breadcrumb={[{ href: resolve('/sessions'), label: 'Sessions' }]}
>
	{#snippet actions()}
		<SessionStepper id={data.session.id} />
	{/snippet}
</PageHeader>

<Grid rowGap>
	<Column narrow lg={11}>
		<TrackMap
			tracks={flights.map((flight) => flight.track)}
			highlight={pointed}
			areas={data.areas}
			token={data.mapboxToken}
			label="Map of the session’s tracks"
		/>
	</Column>
	<!-- On phones below the table, which then follows the map. -->
	<Column lg={5} smOrder={1}>
		<SessionFacts {flights} timeZone={data.timeZone} timeZoneName={data.timeZoneName}>
			{#snippet pilotAction()}
				{#if canChange}
					<IconButton
						icon={Edit}
						label="Change pilot"
						name="Change pilot of the session’s flights"
						onclick={(event) =>
							changePilot(flights, `All ${flightsLabel(flights.length)} in the session.`, event)}
					/>
				{/if}
			{/snippet}
		</SessionFacts>
	</Column>
	<!-- Where the session was flown, before its flights; once the organisation has areas. -->
	{#if data.areas.length > 0}
		<!-- With a gutter, as beside the flight page's facts. -->
		<Column lg={11}>
			<AreaFigures
				{areas}
				airTime={facts.airTime}
				distanceM={flights.reduce((total, flight) => total + flight.distance_m, 0)}
			/>
		</Column>
	{/if}
	<Column narrow>
		<DataTable
			label="Flights of the session"
			{columns}
			rows={flights}
			rowKey={(flight) => flight.id}
			onrowhover={(flight) => (pointed = flight ? flights.indexOf(flight) : null)}
			rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
			selectable={canChange ? () => true : undefined}
			bind:selected
			selectedLabel={(count) => `${count} ${count === 1 ? 'flight' : 'flights'} selected`}
			summary="{flights.length} {flights.length === 1 ? 'flight' : 'flights'}"
		>
			{#snippet batchActions(ids)}
				<Button
					onclick={(event: MouseEvent) =>
						changePilot(
							flights.filter((flight) => ids.includes(flight.id)),
							`The ${flightsLabel(ids.length)} selected.`,
							event
						)}>Change pilot</Button
				>
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
				<Modal
					title="Remove from session"
					submitLabel="Remove"
					action="?/remove"
					error={form?.removeMessage}
					onsuccess={() => removed(ids.length)}
				>
					{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
					<input type="hidden" name="all" value={String(ids.length === flights.length)} />
					<p>
						{#if ids.length === flights.length}
							The session ends. Its {ids.length === 1 ? 'flight stays' : 'flights stay'} in Flights, without
							a session.
						{:else}
							The {ids.length === 1 ? 'flight leaves' : `${ids.length} flights leave`} the session.
							{ids.length === 1 ? 'It stays' : 'They stay'} in Flights, without a session.
						{/if}
					</p>
				</Modal>
			{/snippet}
		</DataTable>
	</Column>
</Grid>

<PilotDialog
	flights={pilotFor}
	which={pilotWhich}
	choices={data.choices}
	bind:open={pilotOpen}
	returnFocus={() => (pilotOpener?.isConnected ? pilotOpener : undefined)}
	error={form?.pilotMessage}
	onsuccess={pilotSet}
/>
