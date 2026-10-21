<!--
	@component
	A flight's or session's time and distance per area, as a small table: most
	time first, then "Outside areas". The rows add up to the totals.
-->
<script lang="ts">
	import { distance, duration } from '#lib/format.js';
	import type { FlightArea } from '#lib/server/backend.js';
	import DataTable, { type Column } from '#lib/ui/DataTable.svelte';

	interface Props {
		/** Most time first. */
		areas: FlightArea[];
		/** The flight's or session's totals. */
		airTime: number;
		distanceM: number;
	}

	let { areas, airTime, distanceM }: Props = $props();

	type Row = FlightArea & { share: string };

	const rows: Row[] = $derived.by(() => {
		const inAreas = areas.reduce((sum, area) => sum + area.air_time_s, 0);
		const outside = airTime - inAreas;
		const distanceInAreas = areas.reduce((sum, area) => sum + area.distance_m, 0);
		const all =
			outside <= 0
				? areas
				: [
						...areas,
						{
							id: 'outside',
							name: 'Outside areas',
							air_time_s: outside,
							distance_m: Math.max(0, distanceM - distanceInAreas)
						}
					];
		return all.map((row) => ({ ...row, share: share(row.air_time_s) }));
	});

	/** Whole percentages; an area the track touched never shows as 0%. */
	function share(seconds: number): string {
		if (airTime <= 0) return '–';
		const percent = Math.round((seconds / airTime) * 100);
		return percent < 1 ? '<1%' : `${percent}%`;
	}

	const columns: Column<Row>[] = [
		{ key: 'name', header: 'Area' },
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (row) => duration(row.air_time_s)
		},
		{ key: 'share', header: 'Share', numeric: true },
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			format: (row) => distance(row.distance_m)
		}
	];
</script>

<section aria-labelledby="areas-heading">
	<h2 id="areas-heading">Areas</h2>
	<DataTable
		label="Time and distance per area"
		{columns}
		{rows}
		rowKey={(row) => row.id}
		size="sm"
	/>
</section>

<style>
	/* In a narrow column the table hangs into the gutter; the heading lines up
	   with the page's text. */
	h2 {
		margin: 0 0 var(--cds-spacing-05);
		padding-inline: var(--cds-spacing-05);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}
</style>
