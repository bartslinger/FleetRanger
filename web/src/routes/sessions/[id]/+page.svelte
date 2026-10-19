<script lang="ts">
	import { resolve } from '$app/paths';
	import { dateTime, day, distance, duration } from '#lib/format.js';
	import SessionFacts, { summary } from '#lib/SessionFacts.svelte';
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

	const facts = $derived(summary(flights, data.timeZone));

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

	function removed(count: number) {
		selected = [];
		toast(`${count} ${count === 1 ? 'flight' : 'flights'} removed from the session.`);
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
	title={`Session ${day(facts.start, data.timeZone, true)}`}
	crumb={`Session ${day(facts.start, data.timeZone)}`}
	description={`${facts.span} · ${flights.length} ${flights.length === 1 ? 'flight' : 'flights'} · ${duration(facts.airTime)} in the air`}
	phoneDescription={`${facts.span} · ${facts.pilots} · ${facts.teams}`}
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
		<SessionFacts {flights} timeZone={data.timeZone} timeZoneName={data.timeZoneName} />
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
			summary="{flights.length} {flights.length === 1 ? 'flight' : 'flights'}"
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
