<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { KINDS, type Kind } from '#lib/equipment.js';
	import { dateTime } from '#lib/format.js';
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

	// Returning to a tab chosen here, SvelteKit loads the page for the URL it
	// was opened with; the tab comes back with the history entry's state.
	// svelte-ignore state_referenced_locally
	let tab: Kind = $state(page.state.equipmentTab ?? data.tab);

	const lastFlight: TableColumn<Row> = $derived({
		key: 'last_flight_at',
		header: 'Last flight',
		format: (item) => item.last_flight_at && dateTime(item.last_flight_at, data.timeZone)
	});

	const columns: Record<Kind, TableColumn<Row>[]> = $derived({
		drones: [
			{ key: 'serial', header: 'Serial number', identifier: true },
			{ key: 'name', header: 'Name' },
			{ key: 'team_name', header: 'Team' },
			{ key: 'holder_name', header: 'Holder' },
			lastFlight
		],
		// As on a controller's page.
		controllers: [
			{ key: 'serial', header: 'Serial number', identifier: true },
			{ key: 'name', header: 'Name' },
			{ key: 'team_name', header: 'Team' },
			{ key: 'holder_name', header: 'Holder' },
			{
				key: 'paired',
				header: 'Pairing',
				format: (item) => (item.paired ? 'Paired' : 'Not paired'),
				// An unpaired controller can't upload; paired is the normal state.
				status: (item) => (item.paired ? undefined : 'warning')
			},
			{
				key: 'synced_at',
				header: 'Last sync',
				format: (item) => (item.synced_at ? dateTime(item.synced_at, data.timeZone) : 'Never')
			},
			lastFlight
		]
	});

	/** The tab is kept in the URL, so the back button and breadcrumbs return to it. */
	function select(next: Kind) {
		replaceState(resolve(next === 'drones' ? '/equipment' : '/equipment?tab=controllers'), {
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
				{ value: 'controllers', label: KINDS.controllers.label }
			]}
			bind:value={tab}
			onchange={select}
		>
			{#snippet panel(kind)}
				{@const rows = kind === 'drones' ? data.drones : data.controllers}
				{#if rows.length === 0}
					<EmptyState
						title={kind === 'drones' ? 'No drones yet' : 'No remote controllers yet'}
						body={data.account?.role === 'member'
							? 'Your team’s drones and remote controllers appear here once an admin assigns them to the team.'
							: kind === 'drones'
								? 'Drones appear here when flight logs that use them are imported.'
								: 'Remote controllers appear here when their flight logs are imported. To add one now, pair it with the code its screen shows.'}
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
		</Tabs>
	</Column>
</Grid>
