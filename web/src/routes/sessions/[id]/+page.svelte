<script lang="ts">
	import { resolve } from '$app/paths';
	import { dateTime, day, distance, duration } from '#lib/format.js';
	import SessionFacts, { summary } from '#lib/SessionFacts.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import Select from '#lib/ui/Select.svelte';
	import Edit from '@carbon/icons/es/edit/16.js';
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

	// The pilot of all its flights, as the flight page sets one flight's.
	const pilotOptions = $derived([
		{ value: '', label: 'Unknown' },
		...data.choices.pilots.map((pilot) => ({
			value: pilot.id,
			label: data.choices.admin ? `${pilot.name} (${pilot.team_name})` : pilot.name
		}))
	]);
	// Their pilot if they share one; "that was me" if none is known yet.
	const pilotChoice = $derived.by(() => {
		const known = new Set(flights.map((flight) => flight.pilot_id));
		if (known.size === 1 && !known.has(null)) return flights[0].pilot_id ?? '';
		if (known.size === 1 && data.choices.pilots.some((pilot) => pilot.id === data.account?.id)) {
			return data.account?.id ?? '';
		}
		return '';
	});
	const severalPilots = $derived(new Set(flights.map((flight) => flight.pilot_id)).size > 1);

	function pilotSet(result: Record<string, unknown> | undefined) {
		const name = (result?.pilotName as string | null) ?? 'Unknown';
		const team = result?.movedTo as string | null | undefined;
		toast(`Pilot set to ${name}.${team ? ` The flights moved to ${team}.` : ''}`);
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
		<SessionFacts {flights} timeZone={data.timeZone} timeZoneName={data.timeZoneName}>
			{#snippet pilotAction()}
				{#if canChange}
					<Modal
						title="Change pilot"
						triggerIcon={Edit}
						submitLabel="Save"
						action="?/setPilot"
						error={form?.pilotMessage}
						onsuccess={pilotSet}
					>
						<input type="hidden" name="team_before" value={flights[0].team_id ?? ''} />
						<Select label="Pilot" name="pilot_id" options={pilotOptions} value={pilotChoice} />
						<p class="help">
							{severalPilots
								? `All ${flights.length} flights get this pilot, also those that have another one now.`
								: `All ${flights.length} flights get this pilot.`}
							{data.choices.admin
								? 'They move to the pilot’s team. With Unknown, they keep their team.'
								: 'Choose Unknown for flights not flown by someone in your team. They stay in your team.'}
						</p>
					</Modal>
				{/if}
			{/snippet}
		</SessionFacts>
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

<style>
	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
