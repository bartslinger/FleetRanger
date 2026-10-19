<!--
	A suggested session, looking like the session it would become: its
	flights on the map, what they add up to, and the flights to leave out.
	Their pilot can be set here already, for all of them or one flight.
	Create session makes it and moves on to the next suggestion, which says
	so until the next step.
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import AddAlt from '@carbon/icons/es/add--alt/16.js';
	import Edit from '@carbon/icons/es/edit/16.js';
	import SubtractAlt from '@carbon/icons/es/subtract--alt/16.js';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { dateTime, day, distance, duration, timeSpan } from '#lib/format.js';
	import SessionFacts, { summary } from '#lib/SessionFacts.svelte';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import IconButton from '#lib/ui/IconButton.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Link from '#lib/ui/Link.svelte';
	import PilotDialog from '#lib/PilotDialog.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Stepper from '#lib/ui/Stepper.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import { query } from '../../logbook.js';
	import type { PageProps, Snapshot } from './$types';

	let { data, form }: PageProps = $props();

	const flights = $derived(data.suggestion.flights);
	type Row = (typeof flights)[number];

	// The flights left out of the session. Another suggestion starts afresh.
	let excluded: string[] = $state([]);
	let stateFor = untrack(() => data.suggestion.key);
	$effect(() => {
		if (data.suggestion.key === stateFor) return;
		stateFor = data.suggestion.key;
		excluded = [];
		selected = [];
	});
	// Back from a flight's page finds the same flights left out.
	export const snapshot: Snapshot<{ key: string; excluded: string[] }> = {
		capture: () => ({ key: data.suggestion.key, excluded }),
		restore: (value) => {
			if (value.key === data.suggestion.key) excluded = value.excluded;
		}
	};

	// Each row's button leaves its flight out or puts it back; the row stays
	// in place, and so does the button, so focus does too.
	function toggle(id: string) {
		excluded = excluded.includes(id) ? excluded.filter((other) => other !== id) : [...excluded, id];
	}

	const chosen = $derived(flights.filter((flight) => !excluded.includes(flight.id)));
	const leftOut = $derived(flights.filter((flight) => excluded.includes(flight.id)));

	// The flight pointed at in the table, brought out on the map.
	let pointed: string | null = $state(null);
	const highlight = $derived.by(() => {
		const index = chosen.findIndex((flight) => flight.id === pointed);
		return index >= 0 ? index : null;
	});
	// The facts of the session it would become.
	const shown = $derived(chosen.length > 0 ? chosen : flights);
	const facts = $derived(summary(shown, data.timeZone));

	// Admins change any flight, members those of their own team.
	const mayChange = (flight: Row) =>
		data.choices.admin ||
		(flight.team_id !== null && data.choices.own_team_ids.includes(flight.team_id));
	const canChange = $derived(chosen.every(mayChange));

	// After creating: the flights left out are a suggestion to look at next,
	// otherwise the next one in the list.
	const then = $derived(leftOut[0]?.id ?? data.suggestion.next ?? data.suggestion.previous ?? '');

	const list = $derived(query(data.state));
	const suggestionHref = (key: string) =>
		`${resolve('/flights/suggestions/[key]', { key })}${list}`;

	const flightsLabel = (count: number) => `${count} ${count === 1 ? 'flight' : 'flights'}`;
	const when = (flight: Row) => dateTime(flight.takeoff_at, data.timeZone);

	// The pilot, of the flights in the session, the selected ones or one
	// flight: one dialog, opened by the pencils and the batch bar.
	let selected: string[] = $state([]);
	let pilotOpen = $state(false);
	let pilotFor: Row[] = $state([]);
	let pilotWhich = $state('');
	let pilotOpener: HTMLElement | undefined;
	function changePilot(of: Row[], which: string, event: Event) {
		pilotFor = of;
		pilotWhich = which;
		pilotOpener = event.currentTarget as HTMLElement;
		pilotOpen = true;
	}
	function pilotSet(result: Record<string, unknown> | undefined) {
		selected = [];
		const id = result?.pilotId as string | null;
		const name = data.choices.pilots.find((pilot) => pilot.id === id)?.name ?? 'Unknown';
		toast(`Pilot set to ${name} for ${flightsLabel(Number(result?.changed ?? 0))}.`);
	}

	// On phones the row's button stays in view: only takeoff and air time beside it.
	const wide = new MediaQuery('min-width: 42rem');
	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: `Time (${data.timeZoneName})`,
			format: (flight) => timeSpan(flight.takeoff_at, flight.landed_at, data.timeZone)
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (flight) => duration(flight.air_time_s)
		},
		...(wide.current
			? ([
					{
						key: 'distance_m',
						header: 'Distance',
						numeric: true,
						format: (flight) => distance(flight.distance_m)
					},
					{ key: 'pilot_name', header: 'Pilot', after: pilotPencil },
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
				] satisfies TableColumn<Row>[])
			: [])
	]);

	// "?/create" alone would drop the list's query (see Modal).
	const action = $derived(`${list || '?'}&/create`);
	let busy = $state(false);
</script>

{#snippet pilotPencil(flight: Row)}
	{#if mayChange(flight)}
		<IconButton
			icon={Edit}
			label="Change pilot"
			name="Change pilot of flight {when(flight)}"
			onclick={(event) => changePilot([flight], `Flight ${when(flight)}.`, event)}
		/>
	{/if}
{/snippet}

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
				<Button type="submit" disabled={busy || chosen.length === 0 || !canChange}
					>Create session</Button
				>
			</form>
		</div>
	{/snippet}
</PageHeader>

<PilotDialog
	flights={pilotFor}
	which={pilotWhich}
	choices={data.choices}
	bind:open={pilotOpen}
	returnFocus={() => (pilotOpener?.isConnected ? pilotOpener : undefined)}
	error={form && 'pilotMessage' in form ? form.pilotMessage : undefined}
	onsuccess={pilotSet}
/>

<Grid rowGap>
	{#if form && 'message' in form}
		<Column narrow>
			<InlineNotification kind="error" title="Session not created"
				>{form.message}</InlineNotification
			>
		</Column>
	{:else if data.created}
		<!-- Until the next step: stepping on drops it from the address. -->
		<Column narrow>
			<InlineNotification kind="success" title="Session created">
				<Link href={resolve('/sessions/[id]', { id: data.created })}>Open session</Link>
			</InlineNotification>
		</Column>
	{/if}
	<!-- Why Create session can't be pressed. -->
	{#if chosen.length === 0}
		<Column narrow>
			<InlineNotification kind="info" title="No flights in the session">
				Put back at least one flight to create it.
			</InlineNotification>
		</Column>
	{:else if !canChange}
		<Column narrow>
			<InlineNotification kind="info" title="You can’t create this session">
				It has flights of another team. Leave them out, or ask an admin.
			</InlineNotification>
		</Column>
	{/if}
	<!-- Left-out flights aren't on the map. -->
	<Column narrow lg={11}>
		<TrackMap
			tracks={chosen.map((flight) => flight.track)}
			{highlight}
			token={data.mapboxToken}
			label="Map of the suggested session’s tracks"
		/>
	</Column>
	<!-- On phones below the table, which then follows the map. -->
	<Column lg={5} smOrder={1}>
		<SessionFacts flights={shown} timeZone={data.timeZone} timeZoneName={data.timeZoneName}>
			{#snippet pilotAction()}
				{#if canChange && chosen.length > 0}
					<IconButton
						icon={Edit}
						label="Change pilot"
						name="Change pilot of the session’s flights"
						onclick={(event) =>
							changePilot(chosen, `All ${flightsLabel(chosen.length)} in the session.`, event)}
					/>
				{/if}
			{/snippet}
		</SessionFacts>
	</Column>
	<!-- Carbon's inline row actions: Remove and Add, circled, as the bare
	     minus read as a dash; a left-out flight stays in its place, marked.
	     They become a suggestion of their own once the session is made. -->
	<Column narrow>
		<DataTable
			label="Flights of the suggested session"
			{columns}
			rows={flights}
			rowKey={(flight) => flight.id}
			rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
			rowState={(flight) => (excluded.includes(flight.id) ? 'Left out' : undefined)}
			onrowhover={(flight) => (pointed = flight?.id ?? null)}
			summary="{chosen.length} of {flightsLabel(flights.length)} in the session"
			selectable={mayChange}
			bind:selected
			selectedLabel={(count) => `${flightsLabel(count)} selected`}
		>
			{#snippet batchActions(ids)}
				<Button
					onclick={(event: MouseEvent) =>
						changePilot(
							flights.filter((flight) => ids.includes(flight.id)),
							`The ${flightsLabel(ids.length)} selected.`,
							event
						)}>Change pilot</Button
				>
			{/snippet}
			{#snippet rowAction(flight)}
				{@const out = excluded.includes(flight.id)}
				<IconButton
					icon={out ? AddAlt : SubtractAlt}
					label={out ? 'Put back' : 'Leave out'}
					name="{out ? 'Put back' : 'Leave out'} flight {when(flight)}"
					size="md"
					onclick={() => toggle(flight.id)}
				/>
			{/snippet}
		</DataTable>
	</Column>
</Grid>

<style>
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--cds-spacing-05);
	}
</style>
