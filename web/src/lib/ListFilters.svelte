<!--
	@component
	The filters above the Flights and Sessions lists: takeoff days, pilots,
	drones, controllers and, for admins, teams. A list adds its own after them.
-->
<script lang="ts" module>
	import { dayMonth } from '#lib/format.js';
	import type { Device, Devices } from '#lib/server/backend.js';

	export interface Filters {
		/** Pilots' user ids, "none" for flights without a pilot; all if empty. */
		pilots: string[];
		/** Drone serials; all if empty. */
		drones: string[];
		/** Controller serials; all if empty. */
		controllers: string[];
		/** Team ids, "none" for no team (the queue); all if empty. */
		teams: string[];
		/** A day ("2026-10-12"), or the first of `to`. */
		from: string | null;
		to: string | null;
	}

	export const NO_FILTERS: Filters = {
		pilots: [],
		drones: [],
		controllers: [],
		teams: [],
		from: null,
		to: null
	};

	export const filtering = (filters: Filters) =>
		filters.pilots.length > 0 ||
		filters.drones.length > 0 ||
		filters.controllers.length > 0 ||
		filters.teams.length > 0 ||
		filters.from !== null;

	/**
	 * The filters' names, for a later page's breadcrumb: "12 Oct–14 Oct",
	 * "Drone-1, Drone-2", …
	 */
	export function filterNames(filters: Filters, devices: Devices): string[] {
		const names = (known: Device[], serials: string[]) =>
			serials
				.map((serial) => known.find((device) => device.serial === serial)?.name ?? serial)
				.join(', ');
		// A day of the URL as "12 Oct": noon UTC lands on that day.
		const short = (day: string) => dayMonth(`${day}T12:00:00Z`, 'UTC');
		return [
			...(filters.from
				? [filters.to ? `${short(filters.from)}–${short(filters.to)}` : short(filters.from)]
				: []),
			...(filters.pilots.length > 0
				? [filters.pilots.map((id) => pilotName(devices, id)).join(', ')]
				: []),
			...(filters.drones.length > 0 ? [names(devices.aircraft, filters.drones)] : []),
			...(filters.controllers.length > 0 ? [names(devices.controllers, filters.controllers)] : []),
			...(filters.teams.length > 0
				? [filters.teams.map((id) => teamName(devices, id)).join(', ')]
				: [])
		];
	}

	function teamName(devices: Devices, id: string) {
		return id === 'none'
			? 'No team (queue)'
			: (devices.teams.find((team) => team.id === id)?.name ?? 'Unknown team');
	}

	function pilotName(devices: Devices, id: string) {
		return id === 'none'
			? 'Unknown pilot'
			: (devices.pilots.find((pilot) => pilot.id === id)?.name ?? 'Unknown user');
	}
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import DateRange from '#lib/ui/DateRange.svelte';
	import MultiSelect from '#lib/ui/MultiSelect.svelte';

	let {
		filters,
		devices,
		admin,
		onchange,
		children
	}: {
		filters: Filters;
		/** The options, narrowed by the other filters. */
		devices: Devices;
		/** Teams are for admins: members see their own teams' anyway. */
		admin: boolean;
		onchange: (change: Partial<Filters>) => void;
		/** The list's own filters, after these. */
		children?: Snippet;
	} = $props();

	// Drones and controllers by name with the serial.
	function deviceOptions(known: Device[], chosen: string[]) {
		const options = known.map((device) => ({
			value: device.serial,
			label: device.name ? `${device.name} (${device.serial})` : device.serial
		}));
		// From a link, e.g. a drone none of whose flights this user sees.
		for (const serial of chosen) {
			if (!known.some((device) => device.serial === serial)) {
				options.push({ value: serial, label: serial });
			}
		}
		return options;
	}
	const droneOptions = $derived(deviceOptions(devices.aircraft, filters.drones));
	const controllerOptions = $derived(deviceOptions(devices.controllers, filters.controllers));
	const pilotOptions = $derived(
		devices.pilots.map((pilot) => ({
			value: pilot.id ?? 'none',
			label: pilot.name ?? 'Unknown pilot'
		}))
	);
	const teamOptions = $derived(
		devices.teams.map((team) => ({
			value: team.id ?? 'none',
			label: team.name ?? 'No team (queue)'
		}))
	);
</script>

<div class="filters">
	<div class="dates">
		<DateRange
			from={filters.from}
			to={filters.to}
			onchange={(from, to) => onchange({ from, to })}
		/>
	</div>
	<MultiSelect
		label="Pilots"
		placeholder="All pilots"
		options={pilotOptions}
		value={filters.pilots}
		onchange={(pilots) => onchange({ pilots })}
	/>
	<MultiSelect
		label="Drones"
		placeholder="All drones"
		options={droneOptions}
		value={filters.drones}
		onchange={(drones) => onchange({ drones })}
	/>
	<MultiSelect
		label="Controllers"
		placeholder="All controllers"
		options={controllerOptions}
		value={filters.controllers}
		onchange={(controllers) => onchange({ controllers })}
	/>
	{#if admin}
		<MultiSelect
			label="Teams"
			placeholder="All teams"
			options={teamOptions}
			value={filters.teams}
			onchange={(teams) => onchange({ teams })}
		/>
	{/if}
	{@render children?.()}
</div>

<style>
	/* Filters above the list: side by side from md, 16px apart, with 16px
	   before the list. Clear filters is in the table's toolbar. */
	.filters {
		display: grid;
		gap: var(--cds-spacing-05);
		align-items: end;
		margin-bottom: var(--cds-spacing-05);
		/* The column is edge to edge on phones; the fields keep the gutter. */
		padding-inline: var(--cds-spacing-05);
	}

	/* From md two columns, the dates across both; from lg one row. */
	@media (min-width: 42rem) {
		.filters {
			padding-inline: 0;
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}

		.dates {
			grid-column: 1 / -1;
		}
	}

	/* From lg a row of fixed widths that wraps onto a second line when the
	   filters don't fit, rather than squeezing them. */
	@media (min-width: 66rem) {
		.filters {
			display: flex;
			flex-wrap: wrap;
		}

		.filters > :global(*) {
			flex: 0 0 13rem;
		}

		.filters > .dates {
			flex-basis: 20rem;
		}
	}
</style>
