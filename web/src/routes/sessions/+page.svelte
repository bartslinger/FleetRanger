<script lang="ts">
	import { resolve } from '$app/paths';
	import { dateTime, distance, duration } from '#lib/format.js';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Row = (typeof data.sessions)[number];

	/** From the first takeoff to the last landing. */
	const span = (session: Row) =>
		(Date.parse(session.last_landed_at) - Date.parse(session.first_takeoff_at)) / 1000;

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'first_takeoff_at',
			header: `Start (${data.timeZoneName})`,
			format: (session) => dateTime(session.first_takeoff_at, data.timeZone)
		},
		{ key: 'last_landed_at', header: 'Duration', numeric: true, format: (s) => duration(span(s)) },
		{ key: 'flights', header: 'Flights', numeric: true },
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (session) => duration(session.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			format: (session) => distance(session.distance_m)
		},
		{ key: 'pilots', header: 'Pilot', format: (session) => session.pilots.join(', ') },
		{ key: 'teams', header: 'Team', format: (session) => session.teams.join(', ') }
	]);

	const href = (page: number, size: number) =>
		`${resolve('/sessions')}${page > 1 || size !== 50 ? `?page=${page}&size=${size}` : ''}`;
</script>

<PageHeader title="Sessions" />

<Grid>
	<Column narrow>
		{#if data.total === 0}
			<EmptyState
				title="No sessions yet"
				body="A pilot’s flights are grouped into sessions once the flights have a pilot. Set pilots in the logbook."
			/>
		{:else}
			<DataTable
				label="Sessions"
				{columns}
				rows={data.sessions}
				rowKey={(session) => session.id}
				rowHref={(session) => resolve('/sessions/[id]', { id: session.id })}
			/>
			<Pagination
				page={data.page}
				size={data.size}
				total={data.total}
				sizes={data.sizes}
				noun="sessions"
				{href}
			/>
		{/if}
	</Column>
</Grid>
