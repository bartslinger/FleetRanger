<!--
	@component
	A flight's facts in groups: who and where, when and how far, and its
	equipment, with Change pilot and Move to group for who may. On the
	flight's page and on Flights with the flight open; the dialogs post to
	the page's own actions with the flight's id, and the toast names what
	the reloaded flight then has.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import Edit from '@carbon/icons/es/edit/16.js';
	import { areaTimes, dateTime, day, distance, duration } from '#lib/format.js';
	import { may } from '#lib/permissions.js';
	import { groupOptions as groupChoices, sharedGroup, showsGroups } from '#lib/groups.js';
	import { offered, pilotOptions as pilotOptions_ } from '#lib/pilots.js';
	import Link from '#lib/ui/Link.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import Select from '#lib/ui/Select.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { Account, FlightChoices, FlightPage } from '#lib/server/backend.js';

	interface Props {
		flight: FlightPage;
		account: Account | null | undefined;
		choices: FlightChoices;
		timeZone: string;
		/** The last action's errors, from the page's `form`. */
		pilotMessage?: string;
		groupMessage?: string;
	}

	let { flight, account, choices, timeZone, pilotMessage, groupMessage }: Props = $props();

	const canSetPilot = $derived(may(account, 'flights.set_pilot', flight.group_id));
	// Its group's own pilots first.
	const pilotOptions = $derived(pilotOptions_(choices, sharedGroup([flight])));
	// "That was me": an unknown pilot starts as the viewer, if they fly.
	const pilotChoice = $derived.by(() => {
		const user = flight.pilot_id ?? account?.id ?? '';
		return offered(choices, user) ? user : '';
	});

	// Moving needs flights.move on its group and the one it goes to.
	const canMove = $derived(may(account, 'flights.move', flight.group_id));
	const groupOptions = $derived(groupChoices(account, 'flights.move', choices.groups));
	function groupSet(result: Record<string, unknown> | undefined) {
		// After the page reloaded its data: the flight as it is now.
		const group = (result?.groupName as string | undefined) ?? flight.group_name;
		toast(`Flight moved to ${group}.`);
	}

	function pilotSet(result: Record<string, unknown> | undefined) {
		const name =
			((result && 'pilotName' in result ? result.pilotName : flight.pilot_name) as string | null) ??
			'Unknown';
		toast(`Pilot set to ${name}.`);
	}
	const time = (iso: string) => dateTime(iso, timeZone);

	/** A name with its serial, the serial alone, or "Unknown". */
	function device(name: string | null, serial: string | null | undefined) {
		if (name && serial) return `${name} (${serial})`;
		return serial ?? 'Unknown';
	}

	type Fact = { label: string; value: string; numeric?: boolean; href?: string };
	const groups: Fact[][] = $derived([
		[
			{ label: 'Pilot', value: flight.pilot_name ?? 'Unknown' },
			...(showsGroups(account) ? [{ label: 'Group', value: flight.group_name }] : []),
			// Whoever sees the flight may open its session; named as its page is.
			{
				label: 'Session',
				value: flight.session_takeoff_at
					? `Session ${day(flight.session_takeoff_at, timeZone, true)}`
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
			...[areaTimes(flight.areas)].filter((fact) => fact !== null),
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
							error={pilotMessage}
							onsuccess={pilotSet}
						>
							<input type="hidden" name="flight_id" value={flight.id} />
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
							error={groupMessage}
							onsuccess={groupSet}
						>
							<input type="hidden" name="flight_id" value={flight.id} />
							<Select
								label="Group"
								name="group_id"
								options={groupOptions}
								value={flight.group_id}
							/>
							<p class="help">The flight keeps its pilot and session.</p>
						</Modal>
					{/if}
				</dd>
			</div>
		{/each}
	</dl>
{/each}

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
