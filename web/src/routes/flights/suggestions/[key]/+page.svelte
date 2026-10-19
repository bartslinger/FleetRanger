<!--
	A suggested session, looking like the session it would become: its
	flights on the map, what they add up to, and the flights to leave out.
	Create session makes it and moves on to the next suggestion, which says
	so until the next step; Create and edit opens the new session.
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { dateTime, day, distance, duration } from '#lib/format.js';
	import SessionFacts, { summary } from '#lib/SessionFacts.svelte';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Link from '#lib/ui/Link.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Stepper from '#lib/ui/Stepper.svelte';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import { query } from '../../logbook.js';
	import type { PageProps, Snapshot } from './$types';

	let { data, form }: PageProps = $props();

	const flights = $derived(data.suggestion.flights);
	type Row = (typeof flights)[number];

	// The flights to make the session of; all of them until some are left out.
	let selected: string[] = $state(untrack(() => flights.map((flight) => flight.id)));
	let selectedFor = untrack(() => data.suggestion.key);
	$effect(() => {
		if (data.suggestion.key === selectedFor) return;
		selectedFor = data.suggestion.key;
		selected = flights.map((flight) => flight.id);
	});
	// Back from a flight's page finds the same flights ticked.
	export const snapshot: Snapshot<{ key: string; selected: string[] }> = {
		capture: () => ({ key: data.suggestion.key, selected }),
		restore: (value) => {
			if (value.key === data.suggestion.key) selected = value.selected;
		}
	};

	const chosen = $derived(flights.filter((flight) => selected.includes(flight.id)));
	// The facts of the session it would become.
	const shown = $derived(chosen.length > 0 ? chosen : flights);
	const facts = $derived(summary(shown, data.timeZone));

	const canChange = $derived(
		data.choices.admin ||
			(data.choices.own_team_id !== null &&
				chosen.every((flight) => flight.team_id === data.choices.own_team_id))
	);

	// After creating: the flights left out are a suggestion to look at next,
	// otherwise the next one in the list.
	const then = $derived(
		flights.find((flight) => !selected.includes(flight.id))?.id ??
			data.suggestion.next ??
			data.suggestion.previous ??
			''
	);

	const list = $derived(query(data.state));
	const suggestionHref = (key: string) =>
		`${resolve('/flights/suggestions/[key]', { key })}${list}`;

	const flightsLabel = (count: number) => `${count} ${count === 1 ? 'flight' : 'flights'}`;

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: `Takeoff (${data.timeZoneName})`,
			format: (flight) => dateTime(flight.takeoff_at, data.timeZone)
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (flight) => duration(flight.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			format: (flight) => distance(flight.distance_m)
		},
		{ key: 'pilot_name', header: 'Pilot' },
		{
			key: 'aircraft_serial',
			header: 'Drone',
			identifier: true,
			format: (flight) => flight.aircraft_name ?? flight.aircraft_serial ?? ''
		},
		{
			key: 'rc_serial',
			header: 'Controller',
			identifier: true,
			format: (flight) => flight.rc_name ?? flight.rc_serial ?? ''
		}
	]);

	// "?/create" alone would drop the list's query (see Modal).
	const action = $derived(`${list || '?'}&/create`);
	let busy = $state(false);
</script>

<PageHeader
	title={`Suggested session ${day(facts.start, data.timeZone, true)}`}
	crumb={`Suggested session ${day(facts.start, data.timeZone)}`}
	status="Suggested"
	description={`${facts.span} · ${flightsLabel(shown.length)} · ${duration(facts.airTime)} in the air`}
	phoneDescription={`${facts.span} · ${facts.pilots} · ${facts.teams}`}
	breadcrumb={[{ href: `${resolve('/flights')}${list}`, label: 'Suggested sessions' }]}
>
	{#snippet actions()}
		<div class="actions">
			<Stepper
				label="Suggested sessions"
				previous={data.suggestion.previous ? suggestionHref(data.suggestion.previous) : null}
				next={data.suggestion.next ? suggestionHref(data.suggestion.next) : null}
				previousLabel="Previous suggestion (newer)"
				nextLabel="Next suggestion (older)"
				position={data.suggestion.position}
				total={data.suggestion.total}
			/>
			<form
				method="POST"
				{action}
				use:enhance={() => {
					busy = true;
					return async ({ result, update }) => {
						busy = false;
						// Created: this suggestion is gone, so Back skips it.
						if (result.type === 'redirect') {
							// eslint-disable-next-line svelte/no-navigation-without-resolve -- the server resolved it.
							await goto(result.location, { replaceState: true });
						} else {
							await update({ reset: false });
						}
					};
				}}
			>
				{#each chosen as flight (flight.id)}
					<input type="hidden" name="flight_id" value={flight.id} />
				{/each}
				<input type="hidden" name="then" value={then} />
				<div class="buttons">
					<Button
						type="submit"
						kind="secondary"
						name="edit"
						value="true"
						disabled={busy || chosen.length === 0 || !canChange}>Create and edit</Button
					>
					<Button
						type="submit"
						name="edit"
						value="false"
						disabled={busy || chosen.length === 0 || !canChange}>Create session</Button
					>
				</div>
			</form>
		</div>
	{/snippet}
</PageHeader>

<Grid rowGap>
	{#if form?.message}
		<Column narrow>
			<InlineNotification kind="error" title={form.message} />
		</Column>
	{:else if data.created}
		<!-- Until the next step: stepping on drops it from the address. -->
		<Column narrow>
			<InlineNotification kind="success" title="Session created.">
				<Link href={resolve('/sessions/[id]', { id: data.created })}>Open session</Link>
			</InlineNotification>
		</Column>
	{/if}
	<Column narrow lg={11}>
		<TrackMap
			tracks={flights.map((flight) => flight.track)}
			token={data.mapboxToken}
			label="Map of the suggested session’s tracks"
		/>
	</Column>
	<Column lg={5}>
		<SessionFacts flights={shown} timeZone={data.timeZone} timeZoneName={data.timeZoneName} />
	</Column>
	<Column narrow>
		<DataTable
			label="Flights of the suggested session"
			{columns}
			rows={flights}
			rowKey={(flight) => flight.id}
			rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
			selectable={() => true}
			bind:selected
			summary="{chosen.length} of {flightsLabel(flights.length)} in the session"
		/>
	</Column>
</Grid>

<style>
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--cds-spacing-05);
	}

	.buttons {
		display: flex;
	}
</style>
