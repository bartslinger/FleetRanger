<script lang="ts">
	import { untrack } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import ListFilters, {
		NO_FILTERS,
		filterNames,
		filtering as listFiltering,
		type Filters
	} from '#lib/ListFilters.svelte';
	import Button from '#lib/ui/Button.svelte';
	import { PAGE_SIZES, PERIOD_SIZE, query, type SessionListState } from './list.js';
	import { clock, day, daySpan, distance, duration, month } from '#lib/format.js';
	import { periodName, periodOf, step, switchTo, today, type Period } from '#lib/period.js';
	import ClickableTile from '#lib/ui/ClickableTile.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import DayStrip from '#lib/ui/DayStrip.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Link from '#lib/ui/Link.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import PeriodNav from '#lib/ui/PeriodNav.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import ViewSwitcher from '#lib/ui/ViewSwitcher.svelte';
	import type { PageProps, Snapshot } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.sessions)[number];

	/** From the first takeoff to the last landing. */
	const span = (session: Row) =>
		(Date.parse(session.last_landed_at) - Date.parse(session.first_takeoff_at)) / 1000;

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'first_takeoff_at',
			header: 'Start',
			format: (session) =>
				`${day(session.first_takeoff_at, data.timeZone, true)}, ${clock(session.first_takeoff_at, data.timeZone)}`
		},
		{ key: 'last_landed_at', header: 'Duration', numeric: true, format: (s) => duration(span(s)) },
		{ key: 'flights', header: 'Flights', numeric: true },
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (session) => duration(session.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			format: (session) => distance(session.distance_m)
		},
		{ key: 'pilots', header: 'Pilot', format: (session) => session.pilots.join(', ') },
		{ key: 'teams', header: 'Team', format: (session) => session.teams.join(', ') }
	]);

	// Merging: admins change any session, members those of their own team.
	let selected: string[] = $state([]);
	const canChange = $derived(data.choices.admin || data.choices.own_team_ids.length > 0);
	const changeable = (session: Row) =>
		data.choices.admin ||
		session.team_ids.every((team) => data.choices.own_team_ids.includes(team));

	function merged(count: number) {
		selected = [];
		toast(`${count} sessions merged.`);
	}

	/** The list's URL; defaults are left out. */
	const href = (state: SessionListState) => `${resolve('/sessions')}${query(state)}`;

	const filtering = $derived(listFiltering(data.state));
	function filter(change: Partial<Filters>) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- href() resolves.
		goto(href({ ...data.state, ...change, page: 1 }), { reset: false });
	}
	// Other filters, another list: the selection no longer means anything.
	const filters = $derived(query({ ...data.state, page: 1, size: PAGE_SIZES[0], view: 'cards' }));
	let selectedFor = untrack(() => filters);
	$effect(() => {
		if (filters === selectedFor) return;
		selectedFor = filters;
		selected = [];
	});

	// In a later page's breadcrumb, a filtered list names its filters.
	const crumb = $derived(['Sessions', ...filterNames(data.state, data.devices)].join(' · '));
	const sessions = (count: number) => `${count} ${count === 1 ? 'session' : 'sessions'}`;

	// Cards, grouped by month; the list comes newest first.
	const months = $derived.by(() => {
		const groups: { name: string; sessions: Row[] }[] = [];
		for (const session of data.sessions) {
			const name = month(session.first_takeoff_at, data.timeZone);
			const last = groups.at(-1);
			if (last?.name === name) last.sessions.push(session);
			else groups.push({ name, sessions: [session] });
		}
		return groups;
	});

	// Exploring: the cards show a week or a month, and from lg a map beside
	// them has all their tracks at once (coarse, from the list). Pointing at
	// a card brings out its session's tracks; nothing loads. Clicking a track
	// marks its card; clicking a card opens the session. Below lg there's no
	// map.
	const wide = new MediaQuery('min-width: 66rem');

	// The map fits the window from where it starts, 1rem above its bottom
	// (user's feedback: a map taller than the window scrolled awkwardly).
	// Above it: the shell's header (3rem), the page header and the 40px
	// controls row, 12.75rem in all, measured at 1056–1920px and with a
	// larger font. More above it (the notice past PERIOD_SIZE sessions)
	// only makes the map a little taller than the window.
	const MAP_HEIGHT = 'calc(100dvh - 12.75rem - var(--cds-spacing-05))';

	// Every flight's track, with the session it's in.
	const flights = $derived(
		data.sessions.flatMap((session) =>
			session.flight_times.flatMap((flight) =>
				flight.track ? [{ session: session.id, track: flight.track }] : []
			)
		)
	);
	const tracks = $derived(
		flights.map(({ track }) => ({ type: 'LineString' as const, coordinates: track }))
	);
	let pointed: string | null = $state(null);
	let marked: string | null = $state(null);
	const highlight = $derived.by(() => {
		const session = pointed ?? marked;
		return flights.flatMap((flight, index) => (flight.session === session ? [index] : []));
	});
	// Another period or other filters: nothing stays marked.
	let markedFor = untrack(() => data.sessions);
	$effect(() => {
		if (data.sessions === markedFor) return;
		markedFor = data.sessions;
		marked = null;
	});
	const cardElements: Record<string, HTMLElement> = $state({});
	function mark(index: number) {
		const session = flights[index]?.session ?? null;
		// The same session again: unmarked.
		marked = session === marked ? null : session;
		if (marked) cardElements[marked]?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
	}

	// Kept in the browser's history, so going back from a session's page
	// restores the marked card and the selection.
	export const snapshot: Snapshot<{ marked: string | null; selected: string[] }> = {
		capture: () => ({ marked, selected }),
		restore: (value) => ({ marked, selected } = value)
	};

	// The period's links: the same filters, another week or month.
	const at = (period: Period) => href({ ...data.state, period });
	const thisPeriod = $derived(periodOf(data.state.period.unit, today(data.timeZone)));
	// In the cards the period isn't a filter: Clear filters leaves it.
	const cardsFiltering = $derived(listFiltering({ ...data.state, from: null }));

	const plural = (count: number, noun: string) => `${count} ${noun}${count === 1 ? '' : 's'}`;
</script>

<PageHeader title="Sessions" {crumb}>
	{#snippet actions()}
		<ViewSwitcher
			label="View"
			options={[
				{
					label: 'Cards',
					href: href({ ...data.state, view: 'cards' }),
					selected: data.state.view === 'cards'
				},
				{
					label: 'Table',
					href: href({ ...data.state, view: 'table' }),
					selected: data.state.view === 'table'
				}
			]}
		/>
	{/snippet}
</PageHeader>

<Grid>
	<Column narrow>
		{#if data.state.view === 'cards'}
			<ListFilters
				filters={data.state}
				none={{ ...data.state, ...NO_FILTERS }}
				devices={data.devices}
				admin={data.choices.admin}
				filtering={cardsFiltering}
				count={sessions(data.total)}
				onchange={filter}
			>
				{#snippet dates()}
					<PeriodNav
						unit={data.state.period.unit}
						units={{
							week: at(switchTo('week', data.state.period, today(data.timeZone))),
							month: at(switchTo('month', data.state.period, today(data.timeZone)))
						}}
						name={periodName(data.state.period)}
						previous={at(step(data.state.period, -1))}
						next={at(step(data.state.period, 1))}
						current={data.state.period.start === thisPeriod.start ? null : at(thisPeriod)}
					/>
				{/snippet}
			</ListFilters>
		{:else if data.total > 0 || filtering}
			<ListFilters
				filters={data.state}
				none={{ ...data.state, ...NO_FILTERS }}
				devices={data.devices}
				admin={data.choices.admin}
				{filtering}
				onchange={filter}
			/>
		{/if}
		{#if data.state.view === 'cards' && data.total === 0}
			<EmptyState
				title="No sessions in {periodName(data.state.period)}"
				body={cardsFiltering
					? 'No sessions you can see in this period have a flight that passes these filters.'
					: 'Step to another period with the arrows, or see all sessions in the table.'}
			>
				{#snippet action()}
					{#if cardsFiltering}
						<Button kind="ghost" onclick={() => filter(NO_FILTERS)}>Clear filters</Button>
					{:else}
						<Link href={href({ ...data.state, view: 'table' })}>Show the table</Link>
					{/if}
				{/snippet}
			</EmptyState>
		{:else if data.total === 0 && filtering}
			<EmptyState
				title="No sessions match"
				body="No sessions you can see have a flight that passes these filters."
			>
				{#snippet action()}
					<Button kind="ghost" onclick={() => filter(NO_FILTERS)}>Clear filters</Button>
				{/snippet}
			</EmptyState>
		{:else if data.total === 0}
			<EmptyState
				title="No sessions yet"
				body="A pilot’s flights are grouped into sessions once the flights have a pilot."
			>
				{#snippet action()}
					<Link href={resolve('/flights')}>Set pilots on the Flights page</Link>
				{/snippet}
			</EmptyState>
		{:else if data.state.view === 'cards'}
			{#if data.total > PERIOD_SIZE}
				<InlineNotification kind="info" title="Only the latest {PERIOD_SIZE}">
					This {data.state.period.unit} has {data.total} sessions; the table shows them all.
				</InlineNotification>
			{/if}
			<div class="cards-view">
				<div class="cards">
					{#each months as group (group.name)}
						<section>
							<!-- Only where the period has two months: else the period's name says it. -->
							{#if months.length > 1}<h2 class="month">{group.name}</h2>{/if}
							<ul>
								{#each group.sessions as session (session.id)}
									<li bind:this={cardElements[session.id]}>
										<ClickableTile
											href={resolve('/sessions/[id]', { id: session.id })}
											marked={wide.current && marked === session.id}
											onpointerenter={() => (pointed = session.id)}
											onpointerleave={() => (pointed = null)}
											onfocus={() => (pointed = session.id)}
											onblur={() => (pointed = null)}
										>
											<span class="card-title">
												{daySpan(session.first_takeoff_at, session.last_landed_at, data.timeZone)}
											</span>
											<span class="card-strip">
												<DayStrip flights={session.flight_times} timeZone={data.timeZone} />
											</span>
											<span class="card-who">
												{session.pilots.join(', ')} · {session.teams.join(', ')}
											</span>
											<span class="card-numbers">
												{plural(session.flights, 'flight')} · {duration(session.air_time_s)} in the air
												· {distance(session.distance_m)}
											</span>
										</ClickableTile>
									</li>
								{/each}
							</ul>
						</section>
					{/each}
				</div>
				{#if wide.current}
					<div class="map-side">
						<!-- The tracks aren't focusable: the cards are the same sessions. -->
						<TrackMap
							{tracks}
							{highlight}
							ends={false}
							height={MAP_HEIGHT}
							ontrackclick={mark}
							token={data.mapboxToken}
							label={`Map of the sessions in ${periodName(data.state.period)}; the list beside it has the same sessions`}
						/>
					</div>
				{/if}
			</div>
		{:else}
			<DataTable
				label="Sessions"
				{columns}
				rows={data.sessions}
				rowKey={(session) => session.id}
				rowHref={(session) => resolve('/sessions/[id]', { id: session.id })}
				selectable={canChange ? changeable : undefined}
				bind:selected
				selectedLabel={(count) => `${sessions(count)} selected`}
				summary={sessions(data.total)}
			>
				{#snippet batchActions(ids)}
					{#if ids.length > 1}
						<Modal
							title="Merge sessions"
							submitLabel="Merge"
							action="?/merge"
							error={form?.mergeMessage}
							onsuccess={() => merged(ids.length)}
						>
							{#each ids as id (id)}<input type="hidden" name="session_id" value={id} />{/each}
							<p>
								The flights of the {ids.length} sessions become one session. Split it on the session’s
								page to undo this.
							</p>
						</Modal>
					{/if}
				{/snippet}
			</DataTable>
			<Pagination
				page={data.state.page}
				size={data.state.size}
				total={data.total}
				sizes={PAGE_SIZES}
				noun="sessions"
				href={(page, size) => href({ ...data.state, page, size })}
			/>
		{/if}
	</Column>
</Grid>

<style>
	/* Each month under its heading, which stays in view while its cards
	   scroll; tiles 2px apart, as in Carbon's tile groups. */
	.cards {
		display: grid;
		gap: var(--cds-spacing-06);
	}

	.month {
		position: sticky;
		top: 3rem;
		z-index: 1;
		margin: 0;
		padding: var(--cds-spacing-03) var(--cds-spacing-05);
		background: var(--cds-background);
		color: var(--cds-text-secondary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.cards ul {
		display: grid;
		gap: var(--cds-spacing-01);
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.card-title {
		display: block;
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.card-who,
	.card-numbers {
		display: block;
		margin-top: var(--cds-spacing-02);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* The last line: room at its end for the tile's arrow (16px + 8px). */
	.card-numbers {
		padding-inline-end: 1.5rem;
		color: var(--cds-text-secondary);
		font-variant-numeric: tabular-nums;
	}

	/* Under the times it pictures, the same width on every card, so the
	   strips line up and share one scale (exploring: the user found a
	   full-width strip more than it needs; beside the title, titles wrapped). */
	.card-strip {
		display: block;
		width: 12rem;
		max-width: 100%;
		margin-block: var(--cds-spacing-03);
	}

	/* Only the cards below lg; from lg, 5 of 16 columns of cards and the map
	   beside them, in view while the cards scroll. */
	.cards-view {
		display: grid;
		gap: var(--cds-spacing-07);
	}

	/* On the page's 16 columns: narrow columns, so 16px apart. */
	@media (min-width: 66rem) {
		.cards-view {
			grid-template-columns: repeat(16, minmax(0, 1fr));
			column-gap: var(--cds-spacing-05);
			align-items: start;
		}

		.cards {
			grid-column: span 5;
		}

		/* Level with the first card, 16px below the header while the cards
		   scroll, and as high as the window allows (carbon-expert's review). */
		.map-side {
			position: sticky;
			top: calc(3rem + var(--cds-spacing-05));
			grid-column: span 11;
		}
	}
</style>
