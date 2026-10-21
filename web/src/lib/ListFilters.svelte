<!--
	@component
	The filters above the Flights and Sessions lists, the same with or without
	a table: in a row, the takeoff days, a Filters button with how many of the
	others are applied, and Clear filters. The button opens a panel with
	pilots, drones, controllers, for admins teams, areas once there are any,
	and the list's own after them; Apply applies them together.
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
		/** Area ids: flights with time in any of them; all if empty. */
		areas: string[];
		/** A day ("2026-10-12"), or the first of `to`. */
		from: string | null;
		to: string | null;
	}

	export const NO_FILTERS: Filters = {
		pilots: [],
		drones: [],
		controllers: [],
		teams: [],
		areas: [],
		from: null,
		to: null
	};

	/** How many of the panel's filters are applied; the dates aren't in it. */
	export const panelCount = (filters: Filters) =>
		[filters.pilots, filters.drones, filters.controllers, filters.teams, filters.areas].filter(
			(values) => values.length > 0
		).length;

	export const filtering = (filters: Filters) =>
		filters.pilots.length > 0 ||
		filters.drones.length > 0 ||
		filters.controllers.length > 0 ||
		filters.teams.length > 0 ||
		filters.areas.length > 0 ||
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
				: []),
			...(filters.areas.length > 0
				? [
						filters.areas
							.map((id) => devices.areas.find((area) => area.id === id)?.name ?? 'Unknown area')
							.join(', ')
					]
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

<script lang="ts" generics="T extends Filters">
	import { untrack, type Snippet } from 'svelte';
	import Button from '#lib/ui/Button.svelte';
	import DateRange from '#lib/ui/DateRange.svelte';
	import FilterPanel from '#lib/ui/FilterPanel.svelte';
	import MultiSelect from '#lib/ui/MultiSelect.svelte';

	// The rule doesn't follow the generic T: `applied` is copied whole into the draft.
	// eslint-disable-next-line svelte/no-unused-props
	let {
		filters: applied,
		none,
		devices,
		admin,
		filtering,
		ownCount = 0,
		onchange,
		own
	}: {
		/** The applied filters, the list's own among them. */
		filters: T;
		/** No filters at all, for Reset and Clear filters. */
		none: T;
		/** The options, narrowed by the applied filters. */
		devices: Devices;
		/** Teams are for admins: members see their own teams' anyway. */
		admin: boolean;
		/** Whether any filter is applied, the list's own too: Clear filters. */
		filtering: boolean;
		/** How many of the list's own filters are applied. */
		ownCount?: number;
		onchange: (change: Partial<T>) => void;
		/** The list's own filters in the panel, after these, on the draft. */
		own?: Snippet<[T, (change: Partial<T>) => void]>;
	} = $props();

	// The panel's choice until Apply; it starts as what's applied.
	let draft: T = $state(untrack(() => ({ ...applied })));
	const change = (next: Partial<T>) => (draft = { ...draft, ...next });
	const filters = $derived(draft);

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
	const areaOptions = $derived(devices.areas.map((area) => ({ value: area.id, label: area.name })));
</script>

<div class="filter-bar">
	<div class="dates">
		<DateRange
			from={applied.from}
			to={applied.to}
			onchange={(from, to) => onchange({ from, to } as Partial<T>)}
		/>
	</div>
	<div class="buttons">
		<FilterPanel
			size="md"
			count={panelCount(applied) + ownCount}
			onopen={() => (draft = { ...applied })}
			onapply={() => onchange(draft)}
			onreset={() => (draft = { ...none, from: applied.from, to: applied.to })}
		>
			<MultiSelect
				label="Pilots"
				placeholder="All pilots"
				options={pilotOptions}
				value={filters.pilots}
				onchange={(pilots) => change({ pilots } as Partial<T>)}
			/>
			<MultiSelect
				label="Drones"
				placeholder="All drones"
				options={droneOptions}
				value={filters.drones}
				onchange={(drones) => change({ drones } as Partial<T>)}
			/>
			<MultiSelect
				label="Controllers"
				placeholder="All controllers"
				options={controllerOptions}
				value={filters.controllers}
				onchange={(controllers) => change({ controllers } as Partial<T>)}
			/>
			{#if admin}
				<MultiSelect
					label="Teams"
					placeholder="All teams"
					options={teamOptions}
					value={filters.teams}
					onchange={(teams) => change({ teams } as Partial<T>)}
				/>
			{/if}
			<!-- Once the organisation has areas: a flight passes when it has time in any of them. -->
			{#if areaOptions.length > 0 || filters.areas.length > 0}
				<MultiSelect
					label="Areas flown"
					placeholder="All areas"
					options={areaOptions}
					value={filters.areas}
					onchange={(areas) => change({ areas } as Partial<T>)}
				/>
			{/if}
			{@render own?.(draft, change)}
		</FilterPanel>
		{#if filtering}
			<Button kind="ghost" size="md" onclick={() => onchange(none)}>Clear filters</Button>
		{/if}
	</div>
</div>

<style>
	/* The dates, then the buttons, with 16px before the list. The column is
	   edge to edge on phones; the row keeps the gutter. Below md the dates
	   take the width and the buttons go under them. */
	.filter-bar {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		align-items: end;
		margin-bottom: var(--cds-spacing-05);
		padding-inline: var(--cds-spacing-05);
	}

	.dates {
		flex: 1 1 100%;
	}

	/* The ghost buttons' text lines up with what's before them: on phones the
	   dates' left edge, from md 16px after the dates. */
	.buttons {
		display: flex;
		gap: var(--cds-spacing-03);
		margin-inline-start: calc(-1 * var(--cds-spacing-05));
	}

	@media (min-width: 42rem) {
		.filter-bar {
			padding-inline: 0;
		}

		.dates {
			flex: 0 0 20rem;
		}
	}
</style>
