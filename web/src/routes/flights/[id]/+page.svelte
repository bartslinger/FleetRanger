<script lang="ts">
	import { resolve } from '$app/paths';
	import { clock, dateTime, dayMonth, distance, duration } from '#lib/format.js';
	import { pilotOptions as pilotOptions_, pilotValue } from '#lib/pilots.js';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Edit from '@carbon/icons/es/edit/16.js';
	import Link from '#lib/ui/Link.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import FlightStepper from './FlightStepper.svelte';
	import Select from '#lib/ui/Select.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const flight = $derived(data.flight);

	// Setting the pilot: admins on any flight, members on their team's.
	const canSetPilot = $derived(
		data.choices.admin ||
			(flight.team_id !== null && data.choices.own_team_ids.includes(flight.team_id))
	);
	const pilotOptions = $derived(pilotOptions_(data.choices));
	// The pilot with the flight's team; "that was me": an unknown pilot starts
	// as the viewer, with the flight's team if they're in it.
	const pilotChoice = $derived.by(() => {
		const user = flight.pilot_id ?? data.account?.id;
		const own = pilotOptions.filter((option) => option.value.startsWith(`${user}:`));
		const value = pilotValue(user ?? null, flight.team_id);
		if (own.some((option) => option.value === value)) return value;
		return flight.pilot_id ? (own[0]?.value ?? '') : '';
	});

	function pilotSet(result: Record<string, unknown> | undefined) {
		const name = (result?.pilotName as string | null) ?? 'Unknown';
		const team = result?.movedTo as string | null | undefined;
		toast(`Pilot set to ${name}.${team ? ` The flight moved to ${team}.` : ''}`);
	}
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
				numeric: true,
				href: flight.aircraft_id
					? resolve('/equipment/[kind=kind]/[id]', { kind: 'drones', id: flight.aircraft_id })
					: undefined
			},
			{
				label: 'Controller',
				value: device(flight.rc_name, flight.rc_serial),
				numeric: true,
				href: flight.rc_id
					? resolve('/equipment/[kind=kind]/[id]', { kind: 'controllers', id: flight.rc_id })
					: undefined
			},
			{ label: 'Battery', value: device(null, flight.metrics.battery_serial), numeric: true },
			{ label: 'Camera', value: device(null, flight.metrics.camera_serial), numeric: true }
		]
	]);
</script>

<PageHeader
	title={`Flight ${dateTime(flight.takeoff_at, data.timeZone)}`}
	description={`${flight.aircraft_name ?? flight.aircraft_serial ?? 'Unknown drone'} · ${flight.pilot_name ?? 'Unknown pilot'}`}
	crumb={`Flight ${dayMonth(flight.takeoff_at, data.timeZone)}, ${clock(flight.takeoff_at, data.timeZone)}`}
	breadcrumb={[{ href: resolve('/flights'), label: 'Flights' }]}
>
	{#snippet actions()}<FlightStepper id={flight.id} />{/snippet}
</PageHeader>

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
						<dd class:numeric={fact.numeric} class:editable={fact.label === 'Pilot' && canSetPilot}>
							<!-- The drone's and controller's pages, for who may see them. -->
							{#if 'href' in fact && fact.href}
								<Link href={fact.href}>{fact.value}</Link>
							{:else}
								{fact.value}
							{/if}
							{#if fact.label === 'Pilot' && canSetPilot}
								<Modal
									title="Change pilot"
									triggerIcon={Edit}
									submitLabel="Save"
									action="?/setPilot"
									error={form?.pilotMessage}
									onsuccess={pilotSet}
								>
									<input type="hidden" name="team_before" value={flight.team_id ?? ''} />
									<Select label="Pilot" name="pilot" options={pilotOptions} value={pilotChoice} />
									<p class="help">
										{data.choices.admin
											? 'The flight moves to the pilot’s team. With Unknown, it keeps its team.'
											: 'Choose Unknown for a flight that wasn’t flown by someone in your team. It stays in your team.'}
									</p>
								</Modal>
							{/if}
						</dd>
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

	/* The pilot with its Change pilot button 8px after it, centred on the line. */
	.editable {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
	}

	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
