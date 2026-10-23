<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { KINDS, lastPilot, type Kind, type Tab } from '#lib/equipment.js';
	import { dateTime, duration, health } from '#lib/format.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Tabs from '#lib/ui/Tabs.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Row = (typeof data.drones)[number];
	type BatteryRow = (typeof data.batteries)[number];

	// Returning to a tab chosen here, SvelteKit loads the page for the URL it
	// was opened with; the tab comes back with the history entry's state.
	// svelte-ignore state_referenced_locally
	let tab: Tab = $state(page.state.equipmentTab ?? data.tab);

	// Over all its flights, also those the user doesn't see: they describe
	// the machine. Carbon has no rule on the order; figures follow the
	// identifiers and status, "Last flown by" beside "Last flight"
	// (carbon-expert's suggestion).
	const figures: TableColumn<Row>[] = $derived([
		{
			key: 'flights',
			header: 'Flights',
			numeric: true,
			format: (item) => item.flights.toLocaleString('en-US')
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (item) => duration(item.air_time_s)
		},
		{ key: 'last_pilot_name', header: 'Last flown by', format: lastPilot },
		{
			key: 'last_flight_at',
			header: 'Last flight',
			format: (item) => item.last_flight_at && dateTime(item.last_flight_at, data.timeZone)
		}
	]);

	const columns: Record<Kind, TableColumn<Row>[]> = $derived({
		drones: [
			{ key: 'serial', header: 'Serial number', identifier: true },
			{ key: 'name', header: 'Name' },
			{ key: 'team_name', header: 'Team' },
			...figures
		],
		// As on a controller's page.
		controllers: [
			{ key: 'serial', header: 'Serial number', identifier: true },
			{ key: 'name', header: 'Name' },
			{ key: 'team_name', header: 'Team' },
			{
				key: 'paired',
				header: 'Pairing',
				format: (item) => (item.paired ? 'Paired' : 'Not paired'),
				// An unpaired controller can't upload; paired is the normal state.
				status: (item) => (item.paired ? undefined : 'warning')
			},
			...figures,
			{
				key: 'synced_at',
				header: 'Last sync',
				format: (item) => (item.synced_at ? dateTime(item.synced_at, data.timeZone) : 'Never')
			}
		]
	});

	// Their state at their latest flight; health is full capacity over design capacity.
	const batteryColumns: TableColumn<BatteryRow>[] = $derived([
		{ key: 'serial', header: 'Serial number', identifier: true },
		{ key: 'name', header: 'Name' },
		{
			key: 'aircraft_serial',
			header: 'Last drone',
			identifier: true,
			format: (item) => item.aircraft_name ?? item.aircraft_serial
		},
		{
			key: 'cycles',
			header: 'Cycles',
			numeric: true,
			format: (item) => item.cycles?.toLocaleString('en-US')
		},
		{
			key: 'full_capacity_mah',
			header: 'Health',
			numeric: true,
			format: (item) => health(item.full_capacity_mah, item.design_capacity_mah)
		},
		{
			key: 'last_flight_at',
			header: 'Last flight',
			format: (item) => dateTime(item.last_flight_at, data.timeZone)
		}
	]);

	/** The tab is kept in the URL, so the back button and breadcrumbs return to it. */
	function select(next: Tab) {
		replaceState(resolve(next === 'drones' ? '/equipment' : `/equipment?tab=${next}`), {
			equipmentTab: next
		});
	}
</script>

<!-- The way to pair a controller, on the controllers' tab only (user's
     request, 2026-10-21; carbon-expert's advice): the table toolbar's primary
     action, as "New …" on Users and Teams, or the empty state's. A link in a
     button's look, as it opens the page that does it. -->
{#snippet pair()}
	<Button href={resolve('/pair')}>Pair a controller</Button>
{/snippet}

<PageHeader title="Equipment" />

<Grid>
	<Column narrow>
		<Tabs
			tabs={[
				{ value: 'drones', label: KINDS.drones.label },
				{ value: 'controllers', label: KINDS.controllers.label },
				{ value: 'batteries', label: 'Batteries' }
			]}
			bind:value={tab}
			onchange={select}
		>
			{#snippet panel(value)}
				{#if value === 'batteries'}
					{@render batteries()}
				{:else}
					{@render equipment(value)}
				{/if}
			{/snippet}
		</Tabs>
	</Column>
</Grid>

{#snippet batteries()}
	{#if data.batteries.length === 0}
		<EmptyState
			title="No batteries yet"
			body={data.account?.role === 'member'
				? 'Batteries appear here once your team flies with them.'
				: 'Batteries appear here when flight logs that use them are imported.'}
		/>
	{:else}
		<DataTable
			label="Batteries"
			columns={batteryColumns}
			rows={data.batteries}
			rowKey={(item) => item.id}
			rowHref={(item) => resolve('/equipment/batteries/[id]', { id: item.id })}
			summary="{data.batteries.length} {data.batteries.length === 1 ? 'battery' : 'batteries'}"
		/>
	{/if}
{/snippet}

{#snippet equipment(kind: Kind)}
	{@const rows = kind === 'drones' ? data.drones : data.controllers}
	{#if rows.length === 0}
		<EmptyState
			title={kind === 'drones' ? 'No drones yet' : 'No remote controllers yet'}
			body={data.account?.role === 'member'
				? 'Drones and remote controllers appear here once your team flies them or an admin assigns them to your team.'
				: kind === 'drones'
					? 'Drones appear here when flight logs that use them are imported.'
					: 'Remote controllers appear here when their flight logs are imported. To add one now, pair it with a pairing code.'}
			action={kind === 'controllers' ? pair : undefined}
		/>
	{:else}
		<DataTable
			label={KINDS[kind].label}
			columns={columns[kind]}
			{rows}
			rowKey={(item) => item.id}
			rowHref={(item) => resolve('/equipment/[kind=kind]/[id]', { kind, id: item.id })}
			toolbar={kind === 'controllers' ? pair : undefined}
			summary="{rows.length} {(rows.length === 1
				? KINDS[kind].one
				: KINDS[kind].label
			).toLowerCase()}"
		/>
	{/if}
{/snippet}
