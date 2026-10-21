<script lang="ts">
	import { untrack } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import { goto, preloadData, pushState, replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import SessionFacts from '#lib/SessionFacts.svelte';
	import type { PageData as SessionData } from './[id]/$types';
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
	import Calendar from '@carbon/icons/es/calendar/16.js';
	import Close from '@carbon/icons/es/close/16.js';
	import ChevronLeft from '@carbon/icons/es/chevron--left/16.js';
	import ChevronRight from '@carbon/icons/es/chevron--right/16.js';
	import { Tooltip } from 'bits-ui';
	import IconButton from '#lib/ui/IconButton.svelte';
	import TooltipContent from '#lib/ui/TooltipContent.svelte';
	import { roving } from '#lib/ui/roving.js';
	import OverflowMenu from '#lib/ui/OverflowMenu.svelte';
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

	// The cards show a week or a month, and from lg a map beside
	// them has all their tracks at once (coarse, from the list). Pointing at
	// a card brings out its session's tracks; nothing loads. Clicking a track
	// marks its card; clicking a card opens the session. Below lg there's no
	// map.
	const wide = new MediaQuery('min-width: 66rem');

	// The map and, a session open, its legend fit the window, 1rem from its
	// bottom (user's feedback: a taller map scrolled awkwardly, a cut-off
	// legend). The same height in both states, so it never resizes; sized
	// for the open one: the shell's header (3rem), the page header with
	// its breadcrumb (8.25rem) above, the legend (1.5rem) below: 13.75rem
	// with the 1rem, measured at 1280px. The list has no breadcrumb, so
	// there it ends 2rem higher.
	const MAP_HEIGHT = 'calc(100dvh - 13.75rem)';

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
	const indexesOf = (session: string | null) =>
		flights.flatMap((flight, index) => (flight.session === session ? [index] : []));

	// From lg a card opens its session over the list, at the session's URL
	// but without loading its page (SvelteKit's shallow routing): the one map
	// stays and moves to the left while it zooms to the session, the cards
	// go, the facts come in on the right and the flights under it. The full
	// data loads meanwhile and fills in. Back closes it; the URL opened
	// directly is the session's own page. Not below lg, or with a modifier
	// key (a new tab).
	const opened = $derived(page.state.session ?? null);
	let loaded = $state<{ id: string; data: SessionData } | null>(null);
	const openedData = $derived(loaded && loaded.id === opened ? loaded.data : null);
	const openedRow = $derived(data.sessions.find((session) => session.id === opened));
	function open(event: MouseEvent, id: string, url: string) {
		if (!wide.current || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) {
			return;
		}
		event.preventDefault();
		pointed = null;
		// The facts at the top, in view; Back returns to this place.
		listScroll = window.scrollY;
		window.scrollTo({ top: 0 });
		show(id, url, pushState);
	}

	/** From the open session to an earlier or later one in the list: in
	 * place of it in the history, so Back and ✕ return to the list. */
	function goToSession(event: MouseEvent, id: string, url: string) {
		if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) return;
		event.preventDefault();
		show(id, url, replaceState);
	}

	function show(id: string, url: string, record: typeof pushState) {
		// The full tracks take over once the map has flown to the session:
		// arriving sooner, they cut the flight short (user's feedback).
		flown = false;
		setTimeout(() => (flown = true), 450);
		record(url, { session: id });
		preloadData(url).then((result) => {
			if (result.type === 'loaded' && result.status === 200) {
				loaded = { id, data: result.data as SessionData };
			}
		});
	}

	// Esc closes the open session, as its ✕ does, unless something over it
	// (a dialog, a menu) takes the key first.
	function closeOnEscape(event: KeyboardEvent) {
		if (event.key !== 'Escape' || !opened || event.defaultPrevented) return;
		if (document.querySelector('[role="dialog"], [role="menu"]')) return;
		history.back();
	}

	// The map: the open session's full tracks once loaded and flown to, else
	// the cards'.
	let flown = $state(true);

	// Where the list was scrolled to when a session opened.
	let listScroll = 0;
	let wasOpen = false;
	$effect(() => {
		if (opened) wasOpen = true;
		else if (wasOpen) {
			wasOpen = false;
			requestAnimationFrame(() => window.scrollTo({ top: listScroll }));
		}
	});
	const full = $derived(openedData && flown ? openedData : null);
	const mapTracks = $derived(full ? full.session.flights.map((flight) => flight.track) : tracks);
	const focus = $derived(opened && !full ? indexesOf(opened) : null);
	// Flying to a session, it's brought out and the rest dimmed.
	const highlight = $derived(full ? [] : opened ? indexesOf(opened) : indexesOf(pointed ?? marked));
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

	// The toolbar's two lines, the name and the count, and which are cut off:
	// a tooltip has those in full, only those (user's feedback). Checked when
	// the width or the text changes.
	// The open session's place in the list (newest first): the earlier one
	// is after it, the later one before.
	const openedIndex = $derived(data.sessions.findIndex((session) => session.id === opened));
	const earlier = $derived(openedRow ? data.sessions[openedIndex + 1] : undefined);
	const later = $derived(openedRow ? data.sessions[openedIndex - 1] : undefined);
	const periodLines = $derived(
		openedRow
			? [
					day(openedRow.first_takeoff_at, data.timeZone, true),
					`${openedIndex + 1} of ${data.total} · ${periodName(data.state.period)}`
				]
			: [periodName(data.state.period), sessions(data.total)]
	);
	let periodHeading: HTMLElement | undefined = $state();
	let cutLines: boolean[] = $state([false, false]);
	const periodCut = $derived(cutLines.some(Boolean));
	const periodLabel = $derived(periodLines.filter((_, line) => cutLines[line]).join(' · '));
	$effect(() => {
		const heading = periodHeading;
		if (!heading) return;
		void periodLines;
		const check = () =>
			(cutLines = [...heading.children].map((line) => line.scrollWidth > line.clientWidth));
		const frame = requestAnimationFrame(check);
		const resizes = new ResizeObserver(check);
		resizes.observe(heading);
		return () => {
			cancelAnimationFrame(frame);
			resizes.disconnect();
		};
	});
	// In the cards the period isn't a filter: Clear filters leaves it.
	const cardsFiltering = $derived(listFiltering({ ...data.state, from: null }));

	const plural = (count: number, noun: string) => `${count} ${noun}${count === 1 ? '' : 's'}`;

	type Flight = SessionData['session']['flights'][number];
	// The open session's flights, as on its page, fewer columns.
	const flightColumns: TableColumn<Flight>[] = $derived([
		{
			key: 'takeoff_at',
			header: 'Time',
			format: (flight) =>
				`${clock(flight.takeoff_at, data.timeZone)}–${clock(flight.landed_at, data.timeZone)}`
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
		{ key: 'aircraft_serial', header: 'Drone serial', identifier: true }
	]);
</script>

<!-- The list's toolbar, attached to the cards as a table's (Carbon's 48px
     toolbar, carbon-expert's check): the period between previous and next,
     with its count; at the end Filters, a dot while some apply, and a menu
     for the unit and the current period, which change less often (no room
     for a switcher in this column). Above the cards, where the pointer
     works (user's choice, 2026-10-21). -->
{#snippet periodToolbar()}
	{@const unit = data.state.period.unit}
	{@const other = unit === 'month' ? 'week' : 'month'}
	<!-- A toolbar for screen readers, one tab stop with the arrow keys
	     between its controls (Carbon's text toolbar, WAI-ARIA). -->
	<div class="list-toolbar" role="toolbar" aria-label="Sessions" use:roving>
		<!-- A session open, the same bar in the same place: the arrows step
		     through the period's sessions instead, in time like the periods
		     (‹ earlier), disabled at its ends; ✕ in place of the list's tools
		     goes back to the list (user's choice, 2026-10-21). -->
		{#if openedRow}
			{@const url = (session: Row) => resolve('/sessions/[id]', { id: session.id })}
			{#if earlier}
				<IconButton
					icon={ChevronLeft}
					size="lg"
					label="Earlier session"
					href={url(earlier)}
					onclick={(event) => goToSession(event, earlier.id, url(earlier))}
				/>
			{:else}
				<IconButton icon={ChevronLeft} size="lg" label="Earlier session" disabled />
			{/if}
			{#if later}
				<IconButton
					icon={ChevronRight}
					size="lg"
					label="Later session"
					href={url(later)}
					onclick={(event) => goToSession(event, later.id, url(later))}
				/>
			{:else}
				<IconButton icon={ChevronRight} size="lg" label="Later session" disabled />
			{/if}
		{:else}
			<IconButton
				icon={ChevronLeft}
				size="lg"
				label="Previous {unit}"
				href={at(step(data.state.period, -1))}
			/>
			<IconButton
				icon={ChevronRight}
				size="lg"
				label="Next {unit}"
				href={at(step(data.state.period, 1))}
			/>
		{/if}
		<!-- After both arrows, as in a calendar's header, so they stay put
		     whatever the name's length (user's feedback). -->
		<!-- Cut off, the full name in a tooltip, on hover and keyboard focus
		     (our rule for truncated values; user's request). -->
		<Tooltip.Root disabled={!periodCut}>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<!-- One of the toolbar's stops only while cut off, so the keyboard
						     reaches the full name too; screen readers read the text. -->
					<h2
						{...props}
						bind:this={periodHeading}
						class="period"
						aria-live="polite"
						data-roving={periodCut ? 'true' : 'false'}
					>
						<span class="period-name">{periodLines[0]}</span>
						<span class="period-count">{periodLines[1]}</span>
					</h2>
				{/snippet}
			</Tooltip.Trigger>
			<TooltipContent side="top">{periodLabel}</TooltipContent>
		</Tooltip.Root>
		<span class="tools">
			{#if openedRow}
				<IconButton
					icon={Close}
					size="lg"
					label="Back to {periodName(data.state.period)}"
					onclick={() => history.back()}
				/>
			{:else}
				<ListFilters
					toolbar
					filters={data.state}
					none={{ ...data.state, ...NO_FILTERS }}
					devices={data.devices}
					admin={data.choices.admin}
					filtering={cardsFiltering}
					onchange={filter}
				/>
				<OverflowMenu
					label="Choose period"
					variant="row"
					icon={Calendar}
					items={[
						{
							label: `Show by ${other}`,
							// eslint-disable-next-line svelte/no-navigation-without-resolve -- at() resolves
							onselect: () => goto(at(switchTo(other, data.state.period, today(data.timeZone))))
						},
						...(data.state.period.start === thisPeriod.start
							? []
							: [
									{
										label: `Go to this ${unit}`,
										// eslint-disable-next-line svelte/no-navigation-without-resolve -- at() resolves
										onselect: () => goto(at(thisPeriod))
									}
								])
					]}
				/>
			{/if}
		</span>
	</div>
{/snippet}

<!-- A session open: its title, the breadcrumb back to the list (through the
     history, so the list is as it was) and no Cards|Table, which has nothing
     to switch there (carbon-expert's advice, user's decision). -->
<svelte:window onkeydown={closeOnEscape} />

<PageHeader
	title={openedRow ? `Session ${day(openedRow.first_takeoff_at, data.timeZone, true)}` : 'Sessions'}
	breadcrumb={openedRow
		? [{ href: href(data.state), label: 'Sessions', follow: () => history.back() }]
		: undefined}
	{crumb}
	reserveBreadcrumb={data.state.view === 'cards' && wide.current}
>
	{#snippet actions()}
		{#if !opened}<ViewSwitcher
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
			/>{/if}
	{/snippet}
</PageHeader>

<Grid>
	<Column narrow>
		{#if data.state.view === 'table' && (data.total > 0 || filtering)}
			<ListFilters
				filters={data.state}
				none={{ ...data.state, ...NO_FILTERS }}
				devices={data.devices}
				admin={data.choices.admin}
				{filtering}
				onchange={filter}
			/>
		{/if}
		{#if data.state.view === 'table' && data.total === 0 && filtering}
			<EmptyState
				title="No sessions match"
				body="No sessions you can see have a flight that passes these filters."
			>
				{#snippet action()}
					<Button kind="ghost" onclick={() => filter(NO_FILTERS)}>Clear filters</Button>
				{/snippet}
			</EmptyState>
		{:else if data.state.view === 'table' && data.total === 0}
			<EmptyState
				title="No sessions yet"
				body="A pilot’s flights are grouped into sessions once the flights have a pilot."
			>
				{#snippet action()}
					<Link href={resolve('/flights')}>Set pilots on the Flights page</Link>
				{/snippet}
			</EmptyState>
		{/if}
		<!-- The map and the cards, also for a period without sessions, so the
		     layout doesn't change (user's feedback): the empty state takes
		     the cards' place, under their toolbar. -->
		{#if data.state.view === 'cards'}
			{#if data.total > PERIOD_SIZE}
				<InlineNotification kind="info" title="Only the latest {PERIOD_SIZE}">
					This {data.state.period.unit} has {data.total} sessions; the table shows them all.
				</InlineNotification>
			{/if}
			<div class="cards-view" class:opened={opened && wide.current}>
				<!-- The cards, and in their place the open session's facts. -->
				<div class="side">
					<!-- Stays when a session opens; only its controls change. -->
					{@render periodToolbar()}
					<div class="cards" inert={opened !== null}>
						{#if data.total === 0}
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
						{/if}
						{#each months as group (group.name)}
							<section>
								<!-- Only where the period has two months: else the period's name says it. -->
								{#if months.length > 1}<h2 class="month">{group.name}</h2>{/if}
								<ul>
									{#each group.sessions as session (session.id)}
										<li bind:this={cardElements[session.id]}>
											<ClickableTile
												href={resolve('/sessions/[id]', { id: session.id })}
												onclick={(event) =>
													open(event, session.id, resolve('/sessions/[id]', { id: session.id }))}
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
													{plural(session.flights, 'flight')} · {duration(session.air_time_s)} in the
													air · {distance(session.distance_m)}
												</span>
											</ClickableTile>
										</li>
									{/each}
								</ul>
							</section>
						{/each}
					</div>
					{#if opened && wide.current}
						<div class="facts-side">
							{#if openedData}
								<SessionFacts flights={openedData.session.flights} timeZone={data.timeZone} />
							{/if}
						</div>
					{/if}
				</div>
				{#if wide.current}
					<div class="map-side">
						<!-- The tracks aren't focusable: the cards are the same sessions. -->
						<TrackMap
							tracks={mapTracks}
							{highlight}
							{focus}
							ends={full !== null}
							height={MAP_HEIGHT}
							ontrackclick={opened ? undefined : mark}
							token={data.mapboxToken}
							label={openedRow
								? 'Map of the session’s tracks'
								: `Map of the sessions in ${periodName(data.state.period)}; the list beside it has the same sessions`}
						/>
					</div>
				{/if}
			</div>
			{#if opened && openedData}
				<div class="opened-flights">
					<DataTable
						label="Flights of the session"
						columns={flightColumns}
						rows={openedData.session.flights}
						rowKey={(flight) => flight.id}
						rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
						summary={plural(openedData.session.flights.length, 'flight')}
					/>
				</div>
			{/if}
		{/if}
		{#if data.state.view === 'table' && data.total > 0}
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
	/* minmax(0, 1fr), here and in the view: as wide as the column, never as
	   the longest content (the toolbar's name), which scrolled the page
	   sideways (user's feedback). */
	.cards {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
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
	   strips line up and share one scale (user's choice, 2026-10-21: a
	   full-width strip was more than it needs; beside the title, titles
	   wrapped). */
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
		grid-template-columns: minmax(0, 1fr);
		gap: var(--cds-spacing-07);
	}

	/* From lg the map on the left, 10 of 16 columns (11 from xlg), and the
	   cards on the right, 6 (5): a card's last line fits beside its arrow.
	   A session open, its facts take the cards' place and the map stays as
	   it is, only flying to the session, so the pointer stays over text
	   (user's choice, 2026-10-21). 16px between, as padding. */
	@media (min-width: 66rem) {
		.cards-view {
			/* minmax(0, …): a column never grows past its share for its content
			   (the map's canvas, a long name), which scrolled the page sideways. */
			grid-template-columns: minmax(0, 10fr) minmax(0, 6fr);
			gap: 0;
			align-items: start;
		}

		.side {
			position: relative;
			min-width: 0;
			padding-inline-start: var(--cds-spacing-05);
		}

		/* Level with the first card, 16px below the header while the cards
		   scroll, and as high as the window allows (carbon-expert's review). */
		.map-side {
			position: sticky;
			top: calc(3rem + var(--cds-spacing-05));
			order: -1;
		}

		/* Carbon's short slide: the cards leave 32px to the left with the
		   exit curve, out of the flow at once so the facts take their place;
		   the facts come from 32px to the right with the entrance curve. */
		.cards {
			transition:
				opacity 240ms cubic-bezier(0.2, 0, 1, 0.9),
				transform 240ms cubic-bezier(0.2, 0, 1, 0.9),
				visibility 0s;
		}

		.opened .cards {
			position: absolute;
			/* Under the toolbar, which stays: 48px and its 2px. */
			inset: calc(var(--cds-spacing-09) + var(--cds-spacing-01)) 0 auto var(--cds-spacing-05);
			visibility: hidden;
			opacity: 0;
			transform: translateX(-2rem);
			transition:
				opacity 240ms cubic-bezier(0.2, 0, 1, 0.9),
				transform 240ms cubic-bezier(0.2, 0, 1, 0.9),
				visibility 0s 240ms;
		}

		/* 16px under the toolbar. */
		.facts-side {
			padding-top: var(--cds-spacing-05);
			animation: 240ms cubic-bezier(0, 0, 0.38, 0.9) 120ms both facts-in;
		}
	}

	@keyframes facts-in {
		from {
			opacity: 0;
			transform: translateX(2rem);
		}
	}

	@media (min-width: 82rem) {
		.cards-view {
			grid-template-columns: minmax(0, 11fr) minmax(0, 5fr);
		}
	}

	/* Carbon's large table toolbar: 48px on the list's layer, 2px above the
	   first card, its buttons full height. */
	.list-toolbar {
		display: flex;
		align-items: center;
		min-height: var(--cds-spacing-09);
		/* 2px to the first card. */
		margin-bottom: var(--cds-spacing-01);
		background: var(--cds-layer-01);
	}

	/* The buttons keep their 48px; the name gives way, its end (the count)
	   cut off with an ellipsis (user's feedback: the arrows were squeezed). */
	.list-toolbar :global(.icon-button) {
		flex: none;
	}

	/* The name, and under it the count, each line the full width so the
	   count isn't cut off behind the name (user's choice, 2026-10-21):
	   18px + 2px + 16px in the 48px bar. */
	.period {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 0.125rem;
		min-width: 0;
		margin: 0;
		padding-inline: var(--cds-spacing-03);
	}

	.period-name,
	.period-count {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.period-name {
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.period-count {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		font-weight: var(--cds-label-01-font-weight);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.tools {
		display: flex;
		flex: none;
		margin-inline-start: auto;
	}

	/* The flights under the map and facts, coming in after the facts. */
	.opened-flights {
		margin-top: var(--cds-spacing-07);
		animation: 240ms cubic-bezier(0, 0, 0.38, 0.9) 360ms both flights-in;
	}

	@keyframes flights-in {
		from {
			opacity: 0;
			transform: translateY(2rem);
		}
	}
</style>
