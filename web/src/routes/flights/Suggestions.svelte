<!--
	@component
	Suggested sessions for the flights without one: one row per suggestion,
	opening onto its flights (Carbon's expandable rows), each with "New
	session". Unticking a flight leaves it out. Suggestions aren't stored, so a
	created session's flights simply drop out of the next suggestions.
-->
<script lang="ts">
	import { tick } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import type { FlightSummary, Suggestion } from '#lib/server/backend.js';
	import Button from '#lib/ui/Button.svelte';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import DataTable, { type Column } from '#lib/ui/DataTable.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { clock, daySpan, distance, duration } from '#lib/format.js';

	interface Props {
		suggestions: Suggestion[];
		total: number;
		timeZone: string;
		/** Whether the user may group this flight. */
		canChange: (flight: FlightSummary) => boolean;
		/** The open suggestions and the flights left out, by suggestion key;
		 * bound, so the page keeps them for Back. */
		expanded?: string[];
		excluded?: Record<string, string[]>;
	}

	let {
		suggestions,
		total,
		timeZone,
		canChange,
		expanded = $bindable([]),
		excluded = $bindable({})
	}: Props = $props();

	interface Row extends Suggestion {
		span: string;
		count: number;
		air_time_s: number;
		pilots: string;
		teams: string;
		controllers: string;
	}

	/** The known values, e.g. pilots' names; "unknown" too if some flights lack one. */
	function names(flights: FlightSummary[], name: (flight: FlightSummary) => string | null) {
		const known = [...new Set(flights.map(name).filter((value) => value !== null))];
		if (known.length > 0 && flights.some((flight) => name(flight) === null)) known.push('unknown');
		return known.join(', ');
	}

	const rows: Row[] = $derived(
		suggestions.map((suggestion) => {
			const { flights } = suggestion;
			return {
				...suggestion,
				span: daySpan(flights[0].takeoff_at, flights.at(-1)!.landed_at, timeZone, wide.current),
				count: flights.length,
				air_time_s: flights.reduce((sum, flight) => sum + flight.air_time_s, 0),
				pilots: names(flights, (flight) => flight.pilot_name),
				teams: names(flights, (flight) => flight.team_name),
				controllers: names(flights, (flight) => flight.rc_name ?? flight.rc_serial)
			};
		})
	);

	// On phones only when, beside the action; the flights show when opened.
	const wide = new MediaQuery('min-width: 42rem');
	const columns: Column<Row>[] = $derived(
		wide.current
			? [
					{ key: 'span', header: 'When' },
					{ key: 'count', header: 'Flights', numeric: true },
					{
						key: 'air_time_s',
						header: 'Air time',
						numeric: true,
						format: (row) => duration(row.air_time_s)
					},
					{ key: 'pilots', header: 'Pilot', wrap: true },
					{ key: 'teams', header: 'Team', wrap: true },
					{ key: 'controllers', header: 'Controller', wrap: true }
				]
			: [{ key: 'span', header: 'When' }]
	);

	// A suggestion keeps its key while its first flight stays, so the flights
	// left out survive reloading the list.
	const included = (row: Suggestion) =>
		row.flights.filter((flight) => !excluded[row.key]?.includes(flight.id));

	function include(row: Suggestion, id: string, on: boolean) {
		const out = excluded[row.key] ?? [];
		excluded[row.key] = on ? out.filter((other) => other !== id) : [...out, id];
	}

	const flightsLabel = (count: number) => `${count} ${count === 1 ? 'flight' : 'flights'}`;

	// "?/newSession" alone would drop the list's query (see Modal).
	const action = $derived(`${page.url.search}&/newSession`);
	let busy = $state('');
	let error = $state('');
	let table: HTMLElement | undefined = $state();
</script>

{#if error}
	<div class="error">
		<InlineNotification kind="error" title={error} />
	</div>
{/if}
<div bind:this={table}>
	<DataTable
		label="Suggested sessions"
		{columns}
		{rows}
		rowKey={(row) => row.key}
		summary="{total} {total === 1 ? 'suggestion' : 'suggestions'}"
		expandable={(row) => row.count > 1}
		expandLabel={(row) => `Flights of ${row.span}`}
		bind:expanded
	>
		{#snippet expansion(row)}
			<ul class="flights">
				{#each row.flights as flight (flight.id)}
					<li>
						<Checkbox
							label="Include in the session"
							hideLabel
							checked={!excluded[row.key]?.includes(flight.id)}
							onchange={(event) => include(row, flight.id, event.currentTarget.checked)}
						/>
						<a href={resolve('/flights/[id]', { id: flight.id })}>
							{clock(flight.takeoff_at, timeZone)}–{clock(flight.landed_at, timeZone)}
						</a>
						<span>
							{duration(flight.air_time_s)} · {distance(flight.distance_m)} ·
							{flight.aircraft_name ?? flight.aircraft_serial ?? 'Unknown drone'} ·
							{flight.pilot_name ?? 'Pilot unknown'}
						</span>
					</li>
				{/each}
			</ul>
		{/snippet}
		{#snippet rowAction(row)}
			{@const chosen = included(row)}
			<form
				method="POST"
				{action}
				use:enhance={() => {
					busy = row.key;
					const index = rows.indexOf(row);
					return async ({ result, update }) => {
						busy = '';
						error = '';
						await update({ reset: false });
						if (result.type === 'success') {
							toast(`New session with ${flightsLabel(chosen.length)}.`);
							delete excluded[row.key];
							// The row is gone: on to the one that took its place.
							await tick();
							const next = table?.querySelectorAll<HTMLElement>('td.action button');
							next?.[Math.min(index, next.length - 1)]?.focus();
						} else if (result.type === 'failure') {
							const message = result.data?.newSessionMessage;
							error = typeof message === 'string' ? message : 'No session was made.';
						}
					};
				}}
			>
				{#each chosen as flight (flight.id)}
					<input type="hidden" name="flight_id" value={flight.id} />
				{/each}
				<Button
					type="submit"
					kind="ghost"
					size="md"
					disabled={busy !== '' || chosen.length === 0 || !chosen.every(canChange)}
					aria-label="New session of {row.span}, {flightsLabel(chosen.length)}"
				>
					New session{chosen.length < row.count ? ` (${chosen.length})` : ''}
				</Button>
			</form>
		{/snippet}
	</DataTable>
</div>

<style>
	.flights {
		display: grid;
		gap: var(--cds-spacing-02);
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		column-gap: var(--cds-spacing-05);
		min-height: var(--cds-size-small);
	}

	a {
		color: var(--cds-link-primary);
		font-variant-numeric: tabular-nums;
		text-decoration: none;
	}

	a:hover {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}

	a:focus-visible {
		outline: 1px solid var(--cds-focus);
	}

	span {
		color: var(--cds-text-secondary);
	}

	.error {
		margin-bottom: var(--cds-spacing-05);
	}

	form {
		display: flex;
		justify-content: flex-end;
	}

	/* From md wide enough for "New session (12)", so the columns don't shift
	   when a flight is left out. On phones the one column takes up the rest. */
	@media (min-width: 42rem) {
		form {
			min-width: 10rem;
		}
	}
</style>
