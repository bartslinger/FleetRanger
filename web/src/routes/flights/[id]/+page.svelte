<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import { areaShares, clock, dateTime, day, dayMonth, distance, duration } from '#lib/format.js';
	import { may } from '#lib/permissions.js';
	import { groupOptions as groupChoices, ORGANISATION, sharedGroup } from '#lib/groups.js';
	import { offered, pilotOptions as pilotOptions_ } from '#lib/pilots.js';
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

	const canSetPilot = $derived(may(data.account, 'flights.set_pilot', flight.group_id));
	const canArchive = $derived(may(data.account, 'flights.archive', flight.group_id));
	// Its group's own pilots first.
	const pilotOptions = $derived(pilotOptions_(data.choices, sharedGroup([flight])));
	// "That was me": an unknown pilot starts as the viewer, if they fly.
	const pilotChoice = $derived.by(() => {
		const user = flight.pilot_id ?? data.account?.id ?? '';
		return offered(data.choices, user) ? user : '';
	});

	// Moving needs flights.move on its group and the one it goes to.
	const canMove = $derived(may(data.account, 'flights.move', flight.group_id));
	const groupOptions = $derived(groupChoices(data.account, 'flights.move', data.choices.groups));
	function groupSet(result: Record<string, unknown> | undefined) {
		const group = (result?.groupName as string | null) ?? ORGANISATION;
		toast(`Flight moved to ${group}.`);
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

	type Fact = { label: string; value: string; numeric?: boolean; href?: string };
	const groups: Fact[][] = $derived([
		[
			{ label: 'Pilot', value: flight.pilot_name ?? 'Unknown' },
			{ label: 'Group', value: flight.group_name ?? ORGANISATION },
			// Whoever sees the flight may open its session; named as its page is.
			{
				label: 'Session',
				value: flight.session_takeoff_at
					? `Session ${day(flight.session_takeoff_at, data.timeZone, true)}`
					: 'None',
				href: flight.session_id
					? `${resolve('/sessions')}?session=${encodeURIComponent(flight.session_id)}`
					: undefined
			}
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
		{#if canArchive && archived}
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
		{:else if canArchive && flight.session_id === null}
			<!-- Only a flight without a session can be archived. -->
			<Modal
				title="Archive"
				triggerKind="tertiary"
				submitLabel="Archive"
				action="?/archive"
				error={form?.archiveMessage}
				onsuccess={() => toast('Flight archived.')}
			>
				<p>
					The flight is left out of Flights and totals, as if it hadn’t happened. You can restore it
					later.
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
								(fact.label === 'Group' && canMove)}
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
									<p class="help">The flight keeps its group.</p>
								</Modal>
							{/if}
							<!-- As Move to group in Flights. -->
							{#if fact.label === 'Group' && canMove}
								<Modal
									title="Move to group"
									triggerIcon={Edit}
									submitLabel="Move"
									action="?/setGroup"
									error={form?.groupMessage}
									onsuccess={groupSet}
								>
									<Select
										label="Group"
										name="group_id"
										options={groupOptions}
										value={flight.group_id ?? ''}
									/>
									<p class="help">The flight keeps its pilot and session.</p>
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
