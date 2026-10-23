<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import { areaShares, clock, dateTime, dayMonth, distance, duration } from '#lib/format.js';
	import { pilotOptions as pilotOptions_ } from '#lib/pilots.js';
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
	const archived = $derived(flight.archived_at !== null);

	// Setting the pilot: admins on any flight, members on their team's.
	const canSetPilot = $derived(
		data.choices.admin ||
			(flight.team_id !== null && data.choices.own_team_ids.includes(flight.team_id))
	);
	const pilotOptions = $derived(pilotOptions_(data.choices));
	// "That was me": an unknown pilot starts as the viewer, if they fly.
	const pilotChoice = $derived.by(() => {
		const user = flight.pilot_id ?? data.account?.id ?? '';
		return pilotOptions.some((option) => option.value === user) ? user : '';
	});

	const teamOptions = $derived([
		...data.choices.teams.map((team) => ({ value: team.id, label: team.name })),
		{ value: '', label: 'No team (organisation queue)' }
	]);
	function teamSet(result: Record<string, unknown> | undefined) {
		const team = (result?.teamName as string | null) ?? 'the organisation queue';
		toast(`Flight moved to ${team}.`);
	}

	function pilotSet(result: Record<string, unknown> | undefined) {
		const name = (result?.pilotName as string | null) ?? 'Unknown';
		toast(`Pilot set to ${name}.`);
	}
	const time = (iso: string) => dateTime(iso, data.timeZone);

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
			// Only when the flight crossed an area.
			...[areaShares(flight.areas, flight.air_time_s)].filter((fact) => fact !== null),
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
			{
				label: 'Battery',
				value: device(flight.battery_name, flight.metrics.battery_serial),
				numeric: true,
				href: flight.battery_id
					? resolve('/equipment/batteries/[id]', { id: flight.battery_id })
					: undefined
			},
			{ label: 'Camera', value: device(null, flight.metrics.camera_serial), numeric: true }
		]
	]);
</script>

<PageHeader
	title={`Flight ${dateTime(flight.takeoff_at, data.timeZone)}`}
	description={`${flight.aircraft_name ?? flight.aircraft_serial ?? 'Unknown drone'} · ${flight.pilot_name ?? 'Unknown pilot'}`}
	crumb={`Flight ${dayMonth(flight.takeoff_at, data.timeZone)}, ${clock(flight.takeoff_at, data.timeZone)}`}
	breadcrumb={[{ href: resolve('/flights'), label: 'Flights' }]}
	status={archived ? 'Archived' : undefined}
>
	{#snippet actions()}
		<!-- Who may set the pilot may archive and restore. -->
		{#if canSetPilot && archived}
			<form
				method="POST"
				action="?/restore"
				use:enhance={() =>
					async ({ result, update }) => {
						await update();
						if (result.type === 'success') toast('Flight restored.');
					}}
			>
				<Button type="submit" kind="tertiary">Restore</Button>
			</form>
		{:else if canSetPilot}
			<Modal
				title="Archive"
				triggerKind="tertiary"
				submitLabel="Archive"
				action="?/archive"
				error={form?.archiveMessage}
				onsuccess={() => toast('Flight archived.')}
			>
				<p>
					The flight is left out of Flights, sessions and totals, as if it hadn’t happened. It
					leaves its session. You can restore it later.
				</p>
			</Modal>
		{/if}
		<FlightStepper id={flight.id} />
	{/snippet}
</PageHeader>

<Grid rowGap>
	<Column narrow lg={11}>
		<TrackMap
			tracks={[flight.track]}
			areas={data.areas}
			token={data.mapboxToken}
			label="Map of the flight’s track"
		/>
	</Column>
	<Column lg={5}>
		{#each groups as facts, index (index)}
			<dl class="facts">
				{#each facts as fact (fact.label)}
					<div>
						<dt>{fact.label}</dt>
						<dd
							class:numeric={fact.numeric}
							class:editable={(fact.label === 'Pilot' && canSetPilot) ||
								(fact.label === 'Team' && data.choices.admin)}
						>
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
									<Select label="Pilot" name="pilot" options={pilotOptions} value={pilotChoice} />
									<p class="help">The flight keeps its team.</p>
								</Modal>
							{/if}
							<!-- Admins only, as Move to team in Flights. -->
							{#if fact.label === 'Team' && data.choices.admin}
								<Modal
									title="Move to team"
									triggerIcon={Edit}
									submitLabel="Move"
									action="?/setTeam"
									error={form?.teamMessage}
									onsuccess={teamSet}
								>
									<Select
										label="Team"
										name="team_id"
										options={teamOptions}
										value={flight.team_id ?? ''}
									/>
									<p class="help">A pilot who isn’t in the new team is cleared.</p>
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
