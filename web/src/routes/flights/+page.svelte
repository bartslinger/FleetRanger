<script lang="ts">
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import ListFilters, {
		NO_FILTERS,
		filterNames,
		filtering as listFiltering,
		type Filters as ListFilterState
	} from '#lib/ListFilters.svelte';
	import OverflowMenu from '#lib/ui/OverflowMenu.svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import Pagination from '#lib/ui/Pagination.svelte';
	import Select from '#lib/ui/Select.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime, distance, duration } from '#lib/format.js';
	import { pilotOptions } from '#lib/pilots.js';
	import { DEFAULT_SORT, PAGE_SIZES, query, type LogbookState } from './logbook.js';
	import SessionChoice from './SessionChoice.svelte';
	import Suggestions from './Suggestions.svelte';
	import ViewSwitcher from '#lib/ui/ViewSwitcher.svelte';
	import type { PageProps, Snapshot } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.flights)[number];

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: 'Takeoff',
			sortable: true,
			format: (flight) => dateTime(flight.takeoff_at, data.timeZone)
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			sortable: true,
			format: (flight) => duration(flight.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			sortable: true,
			format: (flight) => distance(flight.distance_m)
		},
		{ key: 'pilot_name', header: 'Pilot' },
		{ key: 'team_name', header: 'Team' },
		{
			key: 'aircraft_serial',
			header: 'Drone',
			identifier: true,
			sortable: true,
			format: (flight) => flight.aircraft_name ?? flight.aircraft_serial ?? ''
		},
		{
			key: 'rc_serial',
			header: 'Controller',
			identifier: true,
			sortable: true,
			format: (flight) => flight.rc_name ?? flight.rc_serial ?? ''
		}
	]);

	const filtering = $derived(
		listFiltering(data.state) || data.state.withoutSession || data.state.archived
	);
	const sessionOptions = [
		{ value: '', label: 'All' },
		{ value: 'none', label: 'Without session' }
	];
	const statusOptions = [
		{ value: '', label: 'Active' },
		{ value: 'archived', label: 'Archived' }
	];

	// In a later page's breadcrumb, a filtered list names its filters:
	// "Flights · Drone-1, Drone-2".
	const crumb = $derived(
		[
			'Flights',
			...filterNames(data.state, data.devices),
			...(data.state.withoutSession
				? [data.state.suggestions ? 'suggested sessions' : 'without session']
				: []),
			...(data.state.archived ? ['archived'] : [])
		].join(' · ')
	);

	type Filters = ListFilterState & Pick<LogbookState, 'withoutSession' | 'archived'>;
	const none: Filters = { ...NO_FILTERS, withoutSession: false, archived: false };

	function filter(change: Partial<Filters>) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- href() resolves.
		goto(href({ ...data.state, ...change, page: 1 }), { reset: false });
	}

	// Batch actions: admins change any flight, members those of their own team.
	let selected: string[] = $state([]);

	// Kept in the browser's history, so going back from a flight's page
	// restores the selection.
	export const snapshot: Snapshot<{ selected: string[]; allMatching: boolean }> = {
		capture: () => ({ selected, allMatching }),
		restore: (value) => ({ selected, allMatching } = value)
	};

	// All flights that match the filters, across pages, instead of `selected`.
	let allMatching = $state(false);
	/** How many flights a batch action takes. */
	const count = (ids: string[]) => (allMatching ? data.total : ids.length);

	// Other filters, another list: the selection no longer means anything.
	// Paging and sorting keep it.
	const filters = $derived(
		query({ ...data.state, page: 1, size: PAGE_SIZES[0], sort: DEFAULT_SORT })
	);
	let selectedFor = untrack(() => filters);
	$effect(() => {
		if (filters === selectedFor) return;
		selectedFor = filters;
		selected = [];
		allMatching = false;
	});
	const canChange = $derived(data.choices.admin || data.choices.own_team_ids.length > 0);
	const changeable = (flight: Row) =>
		data.choices.admin ||
		(flight.team_id !== null && data.choices.own_team_ids.includes(flight.team_id));

	const pilotChoices = $derived(pilotOptions(data.choices));
	const teamOptions = $derived([
		...data.choices.teams.map((team) => ({ value: team.id, label: team.name })),
		{ value: '', label: 'No team (organisation queue)' }
	]);

	const flights = (count: number) => `${count} ${count === 1 ? 'flight' : 'flights'}`;

	// Dialogs opened from the batch bar's overflow menu, which gets focus back.
	let pilotOpen = $state(false);
	let teamOpen = $state(false);
	let addOpen = $state(false);
	let removeOpen = $state(false);
	let archiveOpen = $state(false);
	let overflow: HTMLElement | null = $state(null);
	const wide = new MediaQuery('min-width: 42rem');

	function done(message: string) {
		selected = [];
		allMatching = false;
		toast(message);
	}

	/** The selected flights on this page, for the sessions near them. */
	const selectedRows = (ids: string[]) => data.flights.filter((flight) => ids.includes(flight.id));

	function href(state: LogbookState) {
		return `${resolve('/flights')}${query(state)}`;
	}
</script>

<!-- A batch action's flights: the selected ones, or all that match the
     filters with the number the user saw (the server checks it). -->
{#snippet chosenFields(ids: string[])}
	{#if allMatching}
		<input type="hidden" name="all" value="true" />
		<input type="hidden" name="expected" value={data.total} />
	{:else}
		{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
	{/if}
{/snippet}

<PageHeader title="Flights" {crumb}>
	{#snippet actions()}
		<!-- The flights without a session, as a list or grouped into suggestions. -->
		{#if data.state.withoutSession}
			<ViewSwitcher
				label="View"
				options={[
					{
						label: 'List',
						href: href({ ...data.state, suggestions: false, page: 1 }),
						selected: !data.state.suggestions
					},
					{
						label: 'Suggestions',
						href: href({ ...data.state, suggestions: true, page: 1 }),
						selected: data.state.suggestions
					}
				]}
			/>
		{/if}
	{/snippet}
</PageHeader>

<Grid>
	<Column narrow>
		{#if data.total > 0 || filtering}
			<ListFilters
				filters={data.state}
				none={{ ...data.state, ...none }}
				devices={data.devices}
				admin={data.choices.admin}
				{filtering}
				ownCount={Number(data.state.withoutSession) + Number(data.state.archived)}
				onchange={filter}
			>
				{#snippet own(draft, change)}
					<!-- Archived flights are never in a session. -->
					{#if !draft.archived}
						<Select
							label="Session"
							options={sessionOptions}
							value={draft.withoutSession ? 'none' : ''}
							onchange={(event) => change({ withoutSession: event.currentTarget.value === 'none' })}
						/>
					{/if}
					<Select
						label="Status"
						options={statusOptions}
						value={draft.archived ? 'archived' : ''}
						onchange={(event) =>
							change({ archived: event.currentTarget.value === 'archived', withoutSession: false })}
					/>
				{/snippet}
			</ListFilters>
		{/if}
		{#if data.total === 0 && filtering}
			<EmptyState title="No flights match" body="No flights you can see pass these filters.">
				{#snippet action()}
					<Button kind="ghost" onclick={() => filter(none)}>Clear filters</Button>
				{/snippet}
			</EmptyState>
		{:else if data.total === 0}
			<EmptyState
				title="No flights yet"
				body="Flights appear here when flight logs from your remote controllers are imported."
			/>
		{:else if data.suggestions}
			<Suggestions
				suggestions={data.suggestions.suggestions}
				total={data.suggestions.total}
				timeZone={data.timeZone}
				canChange={changeable}
				href={(key) => `${resolve('/flights/suggestions/[key]', { key })}${query(data.state)}`}
			/>
			<Pagination
				page={data.state.page}
				size={data.state.size}
				total={data.suggestions.total}
				sizes={PAGE_SIZES}
				noun="suggestions"
				href={(page, size) => href({ ...data.state, page, size })}
			/>
		{:else}
			<DataTable
				label="Flights"
				{columns}
				rows={data.flights}
				rowKey={(flight) => flight.id}
				rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
				sort={data.state.sort}
				defaultSort={DEFAULT_SORT}
				sortHref={(sort) => href({ ...data.state, page: 1, sort: sort ?? DEFAULT_SORT })}
				selectable={canChange ? changeable : undefined}
				bind:selected
				bind:allMatching
				total={data.total}
				selectedLabel={(count, all) =>
					all
						? `All ${count} ${count === 1 ? 'flight' : 'flights'} selected`
						: `${count} ${count === 1 ? 'flight' : 'flights'} selected`}
				selectAllLabel={(total) => `Select all ${total}`}
				summary={flights(data.total)}
			>
				{#snippet batchActions(ids)}
					{#if data.state.archived}
						<!-- Archived flights can only be restored. -->
						<Modal
							title="Restore"
							submitLabel="Restore"
							action="?/restore"
							error={form && 'restoreMessage' in form ? form.restoreMessage : undefined}
							onsuccess={() => done(`${flights(count(ids))} restored.`)}
						>
							{@render chosenFields(ids)}
							<p>
								{count(ids) === 1
									? 'The flight counts again in Flights and totals. It comes back without a session.'
									: `The ${count(ids)} flights count again in Flights and totals. They come back without a session.`}
							</p>
						</Modal>
					{:else}
						<!-- The frequent actions as buttons (on phones only the first); the
					     rest in the overflow menu, as the bar holds two or three. -->
						<Modal
							title="New session"
							submitLabel="Create"
							action="?/newSession"
							error={form && 'newSessionMessage' in form ? form.newSessionMessage : undefined}
							onsuccess={() => done(`New session with ${flights(count(ids))}.`)}
						>
							{@render chosenFields(ids)}
							<p>
								The {flights(count(ids))} become one new session. Flights that are in a session leave
								it.
							</p>
						</Modal>
						<span class="wide-only">
							<Modal
								title="Add to session"
								submitLabel="Add"
								action="?/addToSession"
								error={form && 'addMessage' in form ? form.addMessage : undefined}
								onsuccess={() => done(`${flights(count(ids))} added to the session.`)}
								bind:open={addOpen}
								returnFocus={() => (wide.current ? undefined : (overflow ?? undefined))}
							>
								{@render chosenFields(ids)}
								<SessionChoice flights={selectedRows(ids)} timeZone={data.timeZone} />
								<p class="help">Flights that are in another session leave it.</p>
							</Modal>
						</span>
						<OverflowMenu
							label="More actions"
							variant="batch"
							bind:trigger={overflow}
							items={[
								{ label: 'Add to session', onselect: () => (addOpen = true), phoneOnly: true },
								{ label: 'Remove from session', onselect: () => (removeOpen = true) },
								{ label: 'Set pilot', onselect: () => (pilotOpen = true) },
								...(data.choices.admin
									? [{ label: 'Move to team', onselect: () => (teamOpen = true) }]
									: []),
								{ label: 'Archive', onselect: () => (archiveOpen = true) }
							]}
						/>
						<Modal
							title="Archive"
							submitLabel="Archive"
							action="?/archive"
							error={form && 'archiveMessage' in form ? form.archiveMessage : undefined}
							onsuccess={() => done(`${flights(count(ids))} archived.`)}
							trigger={false}
							bind:open={archiveOpen}
							returnFocus={() => overflow ?? undefined}
						>
							{@render chosenFields(ids)}
							<p>
								{count(ids) === 1
									? 'The flight is left out of Flights, sessions and totals, as if it hadn’t happened. It leaves its session.'
									: `The ${count(ids)} flights are left out of Flights, sessions and totals, as if they hadn’t happened. They leave their sessions.`}
								You can restore archived flights from Flights with Status: Archived.
							</p>
						</Modal>
						<Modal
							title="Remove from session"
							submitLabel="Remove"
							action="?/removeFromSession"
							error={form && 'removeMessage' in form ? form.removeMessage : undefined}
							onsuccess={(result) => {
								const count = Number(result?.removed ?? 0);
								done(
									count === 0
										? 'None of these flights were in a session.'
										: `${flights(count)} removed from ${count === 1 ? 'its session' : 'their sessions'}.`
								);
							}}
							trigger={false}
							bind:open={removeOpen}
							returnFocus={() => overflow ?? undefined}
						>
							{@render chosenFields(ids)}
							<p>
								{count(ids) === 1
									? 'The flight leaves its session. It stays in Flights, without a session.'
									: `The ${count(ids)} flights leave the sessions they are in. They stay in Flights, without a session.`}
							</p>
						</Modal>
						<Modal
							title="Set pilot"
							submitLabel="Set pilot"
							action="?/setPilot"
							error={form && 'pilotMessage' in form ? form.pilotMessage : undefined}
							onsuccess={() => done(`${flights(count(ids))} changed.`)}
							trigger={false}
							bind:open={pilotOpen}
							returnFocus={() => overflow ?? undefined}
						>
							{@render chosenFields(ids)}
							<Select label="Pilot" name="pilot" options={pilotChoices} />
							<p class="help">
								{data.choices.admin
									? 'The flights move to the pilot’s team. With Unknown, they keep their team.'
									: 'Choose Unknown for a flight that wasn’t flown by someone in your team. It stays in your team.'}
							</p>
						</Modal>
						{#if data.choices.admin}
							<Modal
								title="Move to team"
								submitLabel="Move"
								action="?/setTeam"
								error={form && 'teamMessage' in form ? form.teamMessage : undefined}
								onsuccess={() => done(`${flights(count(ids))} changed.`)}
								trigger={false}
								bind:open={teamOpen}
								returnFocus={() => overflow ?? undefined}
							>
								{@render chosenFields(ids)}
								<Select label="Team" name="team_id" options={teamOptions} />
								<p class="help">A pilot who isn’t in the new team is cleared.</p>
							</Modal>
						{/if}
					{/if}
				{/snippet}
			</DataTable>
			<Pagination
				page={data.state.page}
				size={data.state.size}
				total={data.total}
				sizes={PAGE_SIZES}
				noun="flights"
				href={(page, size) => href({ ...data.state, page, size })}
			/>
		{/if}
	</Column>
</Grid>

<style>
	/* Add to session is a button from md; on phones it's in the overflow menu. */
	@media (max-width: 41.99rem) {
		.wide-only {
			display: none;
		}
	}

	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
