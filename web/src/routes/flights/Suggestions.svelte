<!--
	@component
	Suggested sessions for the flights without one: one row per suggestion,
	opening its page (map, flights, and leaving some out), with "New session"
	for one that needs no look. Suggestions aren't stored, so a created
	session's flights simply drop out of the next suggestions.
-->
<script lang="ts">
	import { tick } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import type { FlightSummary, Suggestion } from '#lib/server/backend.js';
	import Button from '#lib/ui/Button.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import DataTable, { type Column } from '#lib/ui/DataTable.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { daySpan, duration } from '#lib/format.js';

	interface Props {
		suggestions: Suggestion[];
		total: number;
		timeZone: string;
		/** Whether the user may group this flight. */
		canChange: (flight: FlightSummary) => boolean;
		/** A suggestion's page. */
		href: (key: string) => string;
	}

	let { suggestions, total, timeZone, canChange, href }: Props = $props();

	interface Row extends Suggestion {
		span: string;
		count: number;
		air_time_s: number;
		pilots: string;
		groups: string;
		controllers: string;
	}

	/** The known values, e.g. pilots' names; "unknown" too if some flights lack one. */
	function names(flights: FlightSummary[], name: (flight: FlightSummary) => string | null) {
		const known = [...new Set(flights.map(name).filter((value) => value !== null))];
		if (known.length > 0 && flights.some((flight) => name(flight) === null)) known.push('unknown');
		return known.join(', ');
	}

	// On phones only when, beside the action; the rest is on the suggestion's page.
	const wide = new MediaQuery('min-width: 42rem');

	const rows: Row[] = $derived(
		suggestions.map((suggestion) => {
			const { flights } = suggestion;
			return {
				...suggestion,
				span: daySpan(flights[0].takeoff_at, flights.at(-1)!.landed_at, timeZone, wide.current),
				count: flights.length,
				air_time_s: flights.reduce((sum, flight) => sum + flight.air_time_s, 0),
				pilots: names(flights, (flight) => flight.pilot_name),
				groups: names(flights, (flight) => flight.group_name),
				controllers: names(flights, (flight) => flight.rc_name ?? flight.rc_serial)
			};
		})
	);

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
					{ key: 'groups', header: 'Group', wrap: true },
					{ key: 'controllers', header: 'Controller', wrap: true }
				]
			: [{ key: 'span', header: 'When' }]
	);

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
		rowHref={(row) => href(row.key)}
		summary="{total} {total === 1 ? 'suggestion' : 'suggestions'}"
	>
		{#snippet rowAction(row)}
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
							toast(`New session with ${flightsLabel(row.count)}.`);
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
				{#each row.flights as flight (flight.id)}
					<input type="hidden" name="flight_id" value={flight.id} />
				{/each}
				<Button
					type="submit"
					kind="ghost"
					size="md"
					disabled={busy !== '' || !row.flights.every(canChange)}
					aria-label="New session of {row.span}, {flightsLabel(row.count)}">New session</Button
				>
			</form>
		{/snippet}
	</DataTable>
</div>

<style>
	.error {
		margin-bottom: var(--cds-spacing-05);
	}

	form {
		display: flex;
		justify-content: flex-end;
	}
</style>
