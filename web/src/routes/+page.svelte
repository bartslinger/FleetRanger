<script lang="ts">
	import { resolve } from '$app/paths';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import { dateTime, distance, duration } from '#lib/format.js';
	import { DEFAULT_SORT, PAGE_SIZES, query, type LogbookState } from './logbook.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

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
		{ key: 'aircraft_serial', header: 'Drone serial', identifier: true, sortable: true },
		{ key: 'rc_serial', header: 'Controller serial', identifier: true, sortable: true }
	]);

	function href(state: LogbookState) {
		return `${resolve('/')}${query(state)}`;
	}
</script>

<PageHeader title="Logbook" />

<Grid>
	<Column narrow>
		{#if data.total === 0}
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
				sort={data.state.sort}
				sortHref={(sort) => href({ ...data.state, page: 1, sort: sort ?? DEFAULT_SORT })}
			/>
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
