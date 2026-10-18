<script lang="ts">
	import { resolve } from '$app/paths';
	import { dateTime, distance, duration } from '#lib/format.js';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const flight = $derived(data.flight);
	const time = (iso: string) => `${dateTime(iso, data.timeZone)} (${data.timeZoneName})`;

	/** A name with its serial, the serial alone, or "Unknown". */
	function device(name: string | null, serial: string | null | undefined) {
		if (name && serial) return `${name} (${serial})`;
		return serial ?? 'Unknown';
	}

	const groups = $derived([
		[
			{ label: 'Pilot', value: flight.pilot_name ?? 'Unknown' },
			{ label: 'Team', value: flight.team_name ?? 'None (organisation queue)' }
		],
		[
			{ label: 'Takeoff', value: time(flight.takeoff_at) },
			{ label: 'Landing', value: time(flight.landed_at) },
			{ label: 'Air time', value: duration(flight.air_time_s), numeric: true },
			{ label: 'Distance', value: distance(flight.distance_m), numeric: true },
			...(flight.metrics.max_altitude_m === undefined
				? []
				: [
						{
							label: 'Highest above takeoff',
							value: `${Math.round(flight.metrics.max_altitude_m)} m`,
							numeric: true
						}
					])
		],
		[
			{
				label: 'Drone',
				value: device(flight.aircraft_name, flight.aircraft_serial),
				numeric: true
			},
			{ label: 'Controller', value: device(flight.rc_name, flight.rc_serial), numeric: true },
			{ label: 'Battery', value: device(null, flight.metrics.battery_serial), numeric: true },
			{ label: 'Camera', value: device(null, flight.metrics.camera_serial), numeric: true }
		]
	]);
</script>

<PageHeader
	title={dateTime(flight.takeoff_at, data.timeZone)}
	breadcrumb={[{ href: resolve('/'), label: 'Logbook' }]}
/>

<Grid rowGap>
	<Column narrow lg={11}>
		<TrackMap tracks={[flight.track]} token={data.mapboxToken} label="Map of the flight’s track" />
	</Column>
	<Column lg={5}>
		{#each groups as facts, index (index)}
			<dl class="facts">
				{#each facts as fact (fact.label)}
					<div>
						<dt>{fact.label}</dt>
						<dd class:numeric={fact.numeric}>{fact.value}</dd>
					</div>
				{/each}
			</dl>
		{/each}
	</Column>
</Grid>

<style>
	/* Related facts 16px apart; 32px between groups. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-07);
	}

	dt {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.numeric {
		font-variant-numeric: tabular-nums slashed-zero;
	}
</style>
