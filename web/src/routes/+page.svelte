<script lang="ts">
	import DataTable, { type Column } from '#lib/ui/DataTable.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import { dateTime, distance, duration } from '#lib/format.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Row = (typeof data.flights)[number];

	const columns: Column<Row>[] = [
		{ key: 'takeoff_at', header: 'Takeoff' },
		{ key: 'air_time_s', header: 'Air time', numeric: true },
		{ key: 'distance_m', header: 'Distance', numeric: true },
		{ key: 'aircraft_serial', header: 'Drone' },
		{ key: 'rc_serial', header: 'Controller' }
	];
</script>

<PageHeader title="Logbook" />

<DataTable {columns} rows={data.flights} rowKey={(flight) => flight.id} empty="No flights yet.">
	{#snippet cell(flight, column)}
		{#if column.key === 'takeoff_at'}
			{dateTime(flight.takeoff_at, data.timeZone)}
		{:else if column.key === 'air_time_s'}
			{duration(flight.air_time_s)}
		{:else if column.key === 'distance_m'}
			{distance(flight.distance_m)}
		{:else}
			{flight[column.key] ?? '–'}
		{/if}
	{/snippet}
</DataTable>
