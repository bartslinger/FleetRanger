<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import Select from '#lib/ui/Select.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime, distance, duration } from '#lib/format.js';
	import { DEFAULT_SORT, PAGE_SIZES, query, type LogbookState } from './logbook.js';
	import SessionChoice from './SessionChoice.svelte';
	import type { PageProps, Snapshot } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.flights)[number];

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: `Takeoff (${data.timeZoneName})`,
			sortable: true,
			format: (flight) => dateTime(flight.takeoff_at, data.timeZone)
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			sortable: true,
			format: (flight) => duration(flight.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			sortable: true,
			format: (flight) => distance(flight.distance_m)
		},
		{ key: 'pilot_name', header: 'Pilot' },
		{ key: 'team_name', header: 'Team' },
		{
			key: 'aircraft_serial',
			header: 'Drone',
			identifier: true,
			sortable: true,
			format: (flight) => flight.aircraft_name ?? flight.aircraft_serial ?? ''
		},
		{
			key: 'rc_serial',
			header: 'Controller',
			identifier: true,
			sortable: true,
			format: (flight) => flight.rc_name ?? flight.rc_serial ?? ''
		}
	]);

	// Filters: a drone and/or controller, by name with the serial.
	type Device = (typeof data.devices.aircraft)[number];
	function deviceOptions(devices: Device[], all: string, current: string | null) {
		const known = devices.map((device) => ({
			value: device.serial,
			label: device.name ? `${device.name} (${device.serial})` : device.serial
		}));
		// From a link, e.g. a drone none of whose flights this user sees.
		if (current && !devices.some((device) => device.serial === current)) {
			known.push({ value: current, label: current });
		}
		return [{ value: '', label: all }, ...known];
	}
	const droneOptions = $derived(
		deviceOptions(data.devices.aircraft, 'All drones', data.state.drone)
	);
	const controllerOptions = $derived(
		deviceOptions(data.devices.controllers, 'All controllers', data.state.controller)
	);
	const filtering = $derived(
		data.state.drone !== null || data.state.controller !== null || data.state.withoutSession
	);
	const sessionOptions = [
		{ value: '', label: 'All' },
		{ value: 'none', label: 'Without session' }
	];

	// In a later page's breadcrumb, a filtered list names its filters:
	// "Flights · Drone-2".
	const name = (devices: Device[], serial: string) =>
		devices.find((device) => device.serial === serial)?.name ?? serial;
	const crumb = $derived(
		[
			'Flights',
			...(data.state.drone ? [name(data.devices.aircraft, data.state.drone)] : []),
			...(data.state.controller ? [name(data.devices.controllers, data.state.controller)] : []),
			...(data.state.withoutSession ? ['without session'] : [])
		].join(' · ')
	);

	function filter(change: Partial<Pick<LogbookState, 'drone' | 'controller' | 'withoutSession'>>) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- href() resolves.
		goto(href({ ...data.state, ...change, page: 1 }), { reset: false });
	}

	// Batch actions: admins change any flight, members those of their own team.
	let selected: string[] = $state([]);

	// Kept in the browser's history, so going back from a flight's page
	// restores the selection.
	export const snapshot: Snapshot<string[]> = {
		capture: () => selected,
		restore: (value) => (selected = value)
	};
	const canChange = $derived(data.choices.admin || data.choices.own_team_id !== null);
	const changeable = (flight: Row) =>
		data.choices.admin || flight.team_id === data.choices.own_team_id;

	const pilotOptions = $derived([
		{ value: '', label: 'Unknown' },
		...data.choices.pilots.map((pilot) => ({
			value: pilot.id,
			label: data.choices.admin ? `${pilot.name} (${pilot.team_name})` : pilot.name
		}))
	]);
	const teamOptions = $derived([
		...data.choices.teams.map((team) => ({ value: team.id, label: team.name })),
		{ value: '', label: 'No team (organisation queue)' }
	]);

	const flights = (count: number) => `${count} ${count === 1 ? 'flight' : 'flights'}`;

	function done(message: string) {
		selected = [];
		toast(message);
	}

	/** The selected flights on this page, for the sessions near them. */
	const selectedRows = (ids: string[]) => data.flights.filter((flight) => ids.includes(flight.id));

	function href(state: LogbookState) {
		return `${resolve('/flights')}${query(state)}`;
	}
</script>

<PageHeader title="Flights" {crumb} />

<Grid>
	<Column narrow>
		{#if data.total > 0 || filtering}
			<div class="filters">
				<Select
					label="Drone"
					options={droneOptions}
					value={data.state.drone ?? ''}
					onchange={(event) => filter({ drone: event.currentTarget.value || null })}
				/>
				<Select
					label="Controller"
					options={controllerOptions}
					value={data.state.controller ?? ''}
					onchange={(event) => filter({ controller: event.currentTarget.value || null })}
				/>
				<Select
					label="Session"
					options={sessionOptions}
					value={data.state.withoutSession ? 'none' : ''}
					onchange={(event) => filter({ withoutSession: event.currentTarget.value === 'none' })}
				/>
				{#if filtering}
					<Button
						kind="ghost"
						size="md"
						onclick={() => filter({ drone: null, controller: null, withoutSession: false })}
					>
						Clear filters
					</Button>
				{/if}
			</div>
		{/if}
		{#if data.total === 0 && filtering}
			<EmptyState title="No flights match" body="No flights you can see pass these filters." />
		{:else if data.total === 0}
			<EmptyState
				title="No flights yet"
				body="Flights appear here when flight logs from your remote controllers are imported."
			/>
		{:else}
			<DataTable
				label="Flights"
				{columns}
				rows={data.flights}
				rowKey={(flight) => flight.id}
				rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
				sort={data.state.sort}
				defaultSort={DEFAULT_SORT}
				sortHref={(sort) => href({ ...data.state, page: 1, sort: sort ?? DEFAULT_SORT })}
				selectable={canChange ? changeable : undefined}
				bind:selected
				selectedLabel={(count) => `${count} ${count === 1 ? 'flight' : 'flights'} selected`}
			>
				{#snippet batchActions(ids)}
					<Modal
						title="Set pilot"
						submitLabel="Set pilot"
						action="?/setPilot"
						error={form && 'pilotMessage' in form ? form.pilotMessage : undefined}
						onsuccess={() => done(`${flights(ids.length)} changed.`)}
					>
						{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
						<Select label="Pilot" name="pilot_id" options={pilotOptions} />
						<p class="help">
							{data.choices.admin
								? 'The flights move to the pilot’s team. With Unknown, they keep their team.'
								: 'Choose Unknown for a flight that wasn’t flown by someone in your team. It stays in your team.'}
						</p>
					</Modal>
					{#if data.choices.admin}
						<Modal
							title="Move to team"
							submitLabel="Move"
							action="?/setTeam"
							error={form && 'teamMessage' in form ? form.teamMessage : undefined}
							onsuccess={() => done(`${flights(ids.length)} changed.`)}
						>
							{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
							<Select label="Team" name="team_id" options={teamOptions} />
							<p class="help">A pilot who isn’t in the new team is cleared.</p>
						</Modal>
					{/if}
					<Modal
						title="New session"
						submitLabel="Create"
						action="?/newSession"
						error={form && 'newSessionMessage' in form ? form.newSessionMessage : undefined}
						onsuccess={() => done(`New session with ${flights(ids.length)}.`)}
					>
						{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
						<p>
							The {flights(ids.length)} become one new session. Flights that are in a session leave it.
						</p>
					</Modal>
					<Modal
						title="Add to session"
						submitLabel="Add"
						action="?/addToSession"
						error={form && 'addMessage' in form ? form.addMessage : undefined}
						onsuccess={() => done(`${flights(ids.length)} added to the session.`)}
					>
						{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
						<SessionChoice flights={selectedRows(ids)} timeZone={data.timeZone} />
						<p class="help">Flights that are in another session leave it.</p>
					</Modal>
				{/snippet}
			</DataTable>
			<Pagination
				page={data.state.page}
				size={data.state.size}
				total={data.total}
				sizes={PAGE_SIZES}
				noun="flights"
				href={(page, size) => href({ ...data.state, page, size })}
			/>
		{/if}
	</Column>
</Grid>

<style>
	/* Filters above the table: side by side from md, 16px apart, with 16px
	   before the table; Clear filters in line with the fields. */
	.filters {
		display: grid;
		gap: var(--cds-spacing-05);
		align-items: end;
		margin-bottom: var(--cds-spacing-05);
		/* The column is edge to edge on phones; the fields keep the gutter. */
		padding-inline: var(--cds-spacing-05);
	}

	@media (min-width: 42rem) {
		.filters {
			padding-inline: 0;
			grid-template-columns: repeat(3, minmax(0, 16rem)) auto;
			justify-content: start;
		}
	}

	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
