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
	import { PAGE_SIZES, query, type SessionListState } from './list.js';
	import { clock, day, daySpan, distance, duration, month } from '#lib/format.js';
	import type { Session } from '#lib/server/backend.js';
	import ClickableTile from '#lib/ui/ClickableTile.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Link from '#lib/ui/Link.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap, { type Track } from '#lib/ui/TrackMap.svelte';
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
			header: `Start (${data.timeZoneName})`,
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

	// From lg, a map beside the cards previews one session: the first, then
	// the one last hovered or focused. Below lg there's no map; a tap opens
	// the session, whose page has the map.
	const wide = new MediaQuery('min-width: 66rem');
	let previewed: string | undefined = $state();
	const shown = $derived(
		data.sessions.find((session) => session.id === previewed) ?? data.sessions[0]
	);
	// Above the map, and in its name for screen readers.
	const previewName = $derived(
		shown
			? `${daySpan(shown.first_takeoff_at, shown.last_landed_at, data.timeZone)} · ${shown.pilots.join(', ')}`
			: ''
	);

	// Hovering previews after a short pause, so moving the pointer across the
	// cards doesn't fetch every session on the way.
	let hoverTimer: ReturnType<typeof setTimeout> | undefined;
	function hover(id: string) {
		clearTimeout(hoverTimer);
		hoverTimer = setTimeout(() => (previewed = id), 150);
	}

	// Kept in the browser's history, so going back from a session's page
	// restores the preview and the selection.
	export const snapshot: Snapshot<{ previewed?: string; selected: string[] }> = {
		capture: () => ({ previewed, selected }),
		restore: (value) => ({ previewed, selected } = value)
	};

	// Tracks are fetched when a session is previewed, and kept while the list
	// stays the same: merging and splitting reload the list and change them.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- a plain store, nothing renders from it
	let cache = new Map<string, Track[]>();
	let cacheOf: Row[] | undefined;
	let shownTracks: Track[] = $state([]);
	let tracksLoading = $state(false);
	let tracksFailed = $state(false);

	$effect(() => {
		if (data.state.view !== 'cards' || !shown || !wide.current) return;
		const id = shown.id;
		if (cacheOf !== data.sessions) {
			cache = new Map();
			cacheOf = data.sessions;
		}
		tracksFailed = false;
		const cached = cache.get(id);
		if (cached) {
			shownTracks = cached;
			tracksLoading = false;
			return;
		}
		let cancelled = false;
		// Never the previous session's tracks under this session's heading;
		// the map keeps its view until the new ones arrive.
		shownTracks = [];
		tracksLoading = true;
		const store = cache;
		fetch(`/api/sessions/${id}`)
			.then((response) => {
				if (!response.ok) throw new Error(String(response.status));
				return response.json() as Promise<Session>;
			})
			.then((session) => {
				const tracks = session.flights.map((flight) => flight.track);
				store.set(id, tracks);
				if (!cancelled) shownTracks = tracks;
			})
			.catch(() => {
				if (!cancelled) tracksFailed = true;
			})
			.finally(() => {
				if (!cancelled) tracksLoading = false;
			});
		return () => {
			cancelled = true;
		};
	});

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
		{#if data.total > 0 || filtering}
			<ListFilters
				filters={data.state}
				none={{ ...data.state, ...NO_FILTERS }}
				devices={data.devices}
				admin={data.choices.admin}
				{filtering}
				onchange={filter}
			/>
		{/if}
		{#if data.total === 0 && filtering}
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
			<!-- As the table's toolbar: how many pass the filters. -->
			{#if filtering}
				<p class="cards-count">{sessions(data.total)}</p>
			{/if}
			<div class="cards-view">
				<div class="cards">
					{#each months as group (group.name)}
						<section>
							<h2 class="month">{group.name}</h2>
							<ul>
								{#each group.sessions as session (session.id)}
									<li>
										<ClickableTile
											href={resolve('/sessions/[id]', { id: session.id })}
											marked={wide.current && session.id === shown?.id}
											onpointerenter={() => hover(session.id)}
											onpointerleave={() => clearTimeout(hoverTimer)}
											onfocus={() => (previewed = session.id)}
										>
											<span class="card-title">
												{daySpan(session.first_takeoff_at, session.last_landed_at, data.timeZone)}
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
						<!-- Not a heading: it changes as the pointer moves over the cards. -->
						<p class="map-heading">{previewName}</p>
						{#if tracksFailed}
							<InlineNotification kind="error" title="The tracks couldn’t load">
								Try again later.
							</InlineNotification>
						{:else}
							<TrackMap
								tracks={shownTracks}
								token={data.mapboxToken}
								label={`Map of the session ${previewName}`}
								loading={tracksLoading}
							/>
						{/if}
					</div>
				{/if}
			</div>
			<div class="cards-pagination">
				<Pagination
					page={data.state.page}
					size={data.state.size}
					total={data.total}
					sizes={PAGE_SIZES}
					noun="sessions"
					href={(page, size) => href({ ...data.state, page, size })}
				/>
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
	/* 32px below the cards, as between the page's rows before. */
	.cards-pagination {
		margin-top: var(--cds-spacing-07);
	}

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

	.card-numbers {
		color: var(--cds-text-secondary);
		font-variant-numeric: tabular-nums;
	}

	/* The number of sessions, like a table's toolbar: 48px high, in the 16px
	   gutter. */
	.cards-count {
		display: flex;
		align-items: center;
		min-height: var(--cds-spacing-09);
		margin: 0 0 var(--cds-spacing-05);
		padding-inline-start: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
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

		.map-side {
			position: sticky;
			top: 3rem;
			grid-column: span 11;
		}
	}

	/* The previewed session, level with the first month's heading. */
	.map-heading {
		margin: 0;
		padding: var(--cds-spacing-03) var(--cds-spacing-05);
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}
</style>
