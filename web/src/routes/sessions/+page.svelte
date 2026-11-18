<script lang="ts">
	import { untrack } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import { afterNavigate, goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { anywhere, may, readsSeveralGroups } from '#lib/permissions.js';
	import { groupOptions as groupChoices, showsGroups, usesAreas } from '#lib/groups.js';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import Select from '#lib/ui/Select.svelte';
	import SessionFacts, { sessionGroup } from '#lib/SessionFacts.svelte';
	import SessionComments from '#lib/SessionComments.svelte';
	import PilotDialog from '#lib/PilotDialog.svelte';
	import Edit from '@carbon/icons/es/edit/16.js';
	import ListFilters, {
		NO_FILTERS,
		filterNames,
		filtering as listFiltering,
		type Filters
	} from '#lib/ListFilters.svelte';
	import Button from '#lib/ui/Button.svelte';
	import { PAGE_SIZES, query, type SessionListState } from './list.js';
	import {
		areaTimes,
		clock,
		day,
		daySpan,
		distance,
		duration,
		month,
		timeSpan
	} from '#lib/format.js';
	import { dayOf, periodName, periodOf, type Period } from '#lib/period.js';
	import ClickableTile from '#lib/ui/ClickableTile.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import DayStrip from '#lib/ui/DayStrip.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Link from '#lib/ui/Link.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import IconButton from '#lib/ui/IconButton.svelte';
	import PeriodToolbar from '#lib/PeriodToolbar.svelte';
	import CardsView from '#lib/CardsView.svelte';
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
		...(showsGroups(data.account) ? [{ key: 'group_name', header: 'Group' } as const] : [])
	]);

	// Merging: sessions.edit on each session's own group.
	let selected: string[] = $state([]);
	const canChange = $derived(anywhere(data.account, 'sessions.edit'));
	const changeable = (session: Row) => may(data.account, 'sessions.edit', session.group_id);

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

	// The map and, a session open, its legend fit the window with the start
	// of the session's flights table under them: its toolbar and header
	// (6rem) after 1rem, so it's seen to be there (user's choice: the
	// flights are a data table, which needs the width). Above: the shell's
	// header (3rem) and the page header with its breadcrumb row (8.25rem);
	// the legend (1.5rem) below the map: 12.75rem, plus the table's 7rem.
	// The same height in both states, so the map never resizes. A banner
	// above the page (such as impersonation) takes its height off too.
	const MAP_HEIGHT = 'calc(100dvh - 19.75rem - var(--banner-height, 0px))';

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
	// The session pointed at: its card (or focused), or a track on the map.
	let pointed: string | null = $state(null);
	let pointedTrack: string | null = $state(null);
	const indexesOf = (session: string | null) =>
		flights.flatMap((flight, index) => (flight.session === session ? [index] : []));

	// A session open over the list, in the URL (`session=`, user's decision,
	// 2026-10-21: one page, so a reload shows the same): a card is a link to
	// it, the layout's load brings it. The one map stays: it flies to the
	// session, the cards go and its facts come in on the right, its flights
	// in the table under it. Back closes it, as ✕ and Esc do.
	const openSession = $derived(data.openSession);
	const opened = $derived(openSession?.id ?? null);
	const openedRow = $derived(data.sessions.find((session) => session.id === opened));
	const sessionHref = (id: string) => `${resolve('/sessions')}${query(data.state, id)}`;
	// A card's second line: its pilots, and its group once there are several.
	const sessionWho = (session: Row) =>
		[session.pilots.join(', '), ...(showsGroups(data.account) ? [sessionGroup(session)] : [])]
			.filter(Boolean)
			.join(' · ');

	// Opened from the list in this tab, Back returns to it as it was (its
	// scroll position too); opened directly, or stepped into another week or
	// month, ✕ replaces it with the list of that one.
	let cameFromList = false;
	const periodParams = (url: URL) =>
		`${url.searchParams.get('week')} ${url.searchParams.get('month')}`;
	afterNavigate(({ from, to }) => {
		if (!to?.url.searchParams.has('session')) cameFromList = false;
		else if (from && periodParams(from.url) !== periodParams(to.url)) cameFromList = false;
		else if (
			from &&
			from.url.pathname === to.url.pathname &&
			!from.url.searchParams.has('session')
		) {
			cameFromList = true;
		}
	});
	function close() {
		if (cameFromList) history.back();
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- href() resolves
		else goto(href(data.state), { replaceState: true });
	}

	// Esc closes the open session, as its ✕ does, unless something over it
	// (a dialog, a menu) takes the key first.
	function closeOnEscape(event: KeyboardEvent) {
		if (event.key !== 'Escape' || !opened || event.defaultPrevented) return;
		if (document.querySelector('[role="dialog"], [role="menu"]')) return;
		close();
	}

	// The map: the open session's tracks, else the cards'. It flies from one
	// to the other (`fly`): to a session, to the next, back to the period.
	const mapTracks = $derived(
		openSession ? openSession.flights.map((flight) => flight.track) : tracks
	);
	// A session open, the flight pointed at in its table or on the map;
	// else the card's.
	let pointedFlight: number | null = $state(null);
	let pointedFlightTrack: number | null = $state(null);
	const highlight = $derived.by(() => {
		if (!openSession) return indexesOf(pointed ?? pointedTrack);
		const flight = pointedFlight ?? pointedFlightTrack;
		return flight === null ? [] : [flight];
	});
	// A flight's track, as its row (user's request, 2026-10-21): pointing at
	// it brings the flight out, its row in the hover state, and its times
	// and air time in a tooltip, as on the cards' day strips; with more than
	// one pilot in the session, whose flight it was (user's request).
	const manyPilots = $derived(
		new Set(openSession?.flights.map((flight) => flight.pilot_id)).size > 1
	);
	const flightTip = (index: number) => {
		const flight = openSession?.flights[index];
		if (!flight) return '';
		const times = `${clock(flight.takeoff_at, data.timeZone)}–${clock(flight.landed_at, data.timeZone)}`;
		const tip = `${times} · ${duration(flight.air_time_s)}`;
		return manyPilots ? `${tip} · ${flight.pilot_name ?? 'Unknown'}` : tip;
	};
	// A track on the map is its session, as its card is (carbon-expert's
	// advice, 2026-10-21): pointing at it brings the session out and its card
	// in its hover state, without scrolling the list; a click opens it.
	const sessionOf = (index: number) => data.sessions.find((s) => s.id === flights[index]?.session);
	function openTrack(index: number) {
		const session = sessionOf(index);
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- sessionHref resolves
		if (session) goto(sessionHref(session.id));
	}
	const trackTip = (index: number) => {
		const session = sessionOf(index);
		if (!session) return '';
		const when = daySpan(session.first_takeoff_at, session.last_landed_at, data.timeZone);
		return `${when} · ${plural(session.flights, 'flight')}`;
	};

	// Kept in the browser's history, so going back restores the selection.
	export const snapshot: Snapshot<{ selected: string[] }> = {
		capture: () => ({ selected }),
		restore: (value) => ({ selected } = value)
	};

	// The period's links: the same filters, another week or month.
	const at = (period: Period) => href({ ...data.state, period });
	// The arrows skip weeks and months without sessions and stop at the
	// logbook's ends (user's request, 2026-11-18): the nearest periods with
	// sessions that pass the filters.
	const periodHref = (start: string | null) =>
		start ? at(periodOf(data.state.period.unit, dayOf(start, data.timeZone))) : null;
	const previousHref = $derived(periodHref(data.around.before));
	const nextHref = $derived(periodHref(data.around.after));

	// The open session's place in the list (newest first).
	const openedIndex = $derived(data.sessions.findIndex((session) => session.id === opened));
	// The sessions before and after it with the list's filters, also in
	// another week or month (user's decision, 2026-10-21), shown in theirs.
	const neighbourHref = (id: string | null, start: string | null) =>
		id && start
			? `${resolve('/sessions')}${query(
					{
						...data.state,
						period: periodOf(data.state.period.unit, dayOf(start, data.timeZone))
					},
					id
				)}`
			: null;
	const earlierHref = $derived(
		neighbourHref(data.neighbours?.next ?? null, data.neighbours?.next_start ?? null)
	);
	const laterHref = $derived(
		neighbourHref(data.neighbours?.previous ?? null, data.neighbours?.previous_start ?? null)
	);
	// The toolbar's two lines, the name and the count.
	// Its day, from its flights: it may be outside the list's period.
	const openedDay = $derived(
		openSession?.flights[0] ? day(openSession.flights[0].takeoff_at, data.timeZone, true) : ''
	);
	const periodLines: [string, string] = $derived(
		openSession
			? [
					openedDay,
					// Counted in time, as the arrows step (the list is newest first):
					// the newest is the last (user's feedback).
					openedRow
						? `${data.total - openedIndex} of ${data.total} · ${periodName(data.state.period)}`
						: periodName(data.state.period)
				]
			: [periodName(data.state.period), sessions(data.total)]
	);
	// In the cards the period isn't a filter: Clear filters leaves it.
	const cardsFiltering = $derived(listFiltering({ ...data.state, from: null }));

	const plural = (count: number, noun: string) => `${count} ${noun}${count === 1 ? '' : 's'}`;

	// The open session's flights: changing their pilot, moving some to a
	// session of their own, taking them out (its page's tasks before).
	type Flight = NonNullable<typeof data.openSession>['flights'][number];
	const openFlights = $derived(openSession?.flights ?? []);
	let selectedFlights: string[] = $state([]);
	// Another session open starts without a selection.
	let selectedFlightsFor = untrack(() => opened);
	$effect(() => {
		if (opened === selectedFlightsFor) return;
		selectedFlightsFor = opened;
		selectedFlights = [];
	});
	// Their pilot: flights.set_pilot on every flight's group. Which flights
	// are in the session: sessions.edit on its own group.
	const canSetPilots = $derived(
		openFlights.every((flight) => may(data.account, 'flights.set_pilot', flight.group_id))
	);
	const canEditOpen = $derived(
		openSession !== null && may(data.account, 'sessions.edit', openSession.group_id)
	);
	const flightsLabel = (count: number) => plural(count, 'flight');
	// Its flights may include some the user doesn't see, so no count of them.
	const deleteConsequences = $derived.by(() => {
		const comments = openSession?.comments.length ?? 0;
		const lost =
			comments === 0
				? ''
				: `, and ${comments === 1 ? 'its comment is' : `its ${comments} comments are`} deleted`;
		return `Session ${openedDay} is deleted. Its flights stay in Flights, without a session${lost}. This can’t be undone.`;
	});

	// Moving the open session to another group: sessions.move on both, and
	// to move its flights along, flights.move on theirs and the new one.
	const canMoveSession = $derived(
		openSession !== null &&
			showsGroups(data.account) &&
			may(data.account, 'sessions.move', openSession.group_id)
	);
	// The chosen group, starting at the session's own.
	let moveTo = $derived(openSession?.group_id ?? '');
	const moveOptions = $derived.by(() => {
		const options = groupChoices(data.account, 'sessions.move', data.choices.groups);
		const current = openSession?.group_id ?? '';
		return options.some((option) => option.value === current)
			? options
			: [...options, { value: current, label: openSession ? sessionGroup(openSession) : '' }];
	});
	const canMoveFlights = $derived(
		openFlights.every((flight) => may(data.account, 'flights.move', flight.group_id)) &&
			may(data.account, 'flights.move', moveTo)
	);
	/** The flights the user sees that aren't in the chosen group yet. */
	const othersThere = $derived(
		openFlights.filter((flight) => flight.group_id !== (moveTo || null)).length
	);
	function sessionMoved(result: Record<string, unknown> | undefined) {
		const name = moveOptions.find((option) => option.value === moveTo)?.label ?? '';
		toast(
			result?.moved ? `Session and its flights moved to ${name}.` : `Session moved to ${name}.`
		);
	}
	function moved(count: number) {
		selectedFlights = [];
		toast(`${flightsLabel(count)} moved to a new session.`);
	}
	function removed(count: number) {
		selectedFlights = [];
		toast(`${flightsLabel(count)} removed from the session.`);
	}
	// The pilot of all its flights or the selected ones, in one dialog.
	let pilotOpen = $state(false);
	let pilotFor: Flight[] = $state([]);
	let pilotWhich = $state('');
	let pilotOpener: HTMLElement | undefined;
	function changePilot(of: Flight[], which: string, event: Event) {
		pilotFor = of;
		pilotWhich = which;
		pilotOpener = event.currentTarget as HTMLElement;
		pilotOpen = true;
	}
	function pilotSet(result: Record<string, unknown> | undefined) {
		selectedFlights = [];
		const id = result?.pilotId as string | null;
		const name = data.choices.pilots.find((pilot) => pilot.id === id)?.name ?? 'Unknown';
		toast(`Pilot set to ${name} for ${flightsLabel(Number(result?.changed ?? 0))}.`);
	}
	const flightColumns: TableColumn<Flight>[] = $derived([
		{
			key: 'takeoff_at',
			header: 'Time',
			format: (flight) => timeSpan(flight.takeoff_at, flight.landed_at, data.timeZone)
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
		// Only where areas are used and one of the flights crossed one.
		...(usesAreas(data.account) && openFlights.some((flight) => flight.areas.length > 0)
			? [
					{
						key: 'areas',
						header: 'Areas',
						wrap: true,
						format: (flight) => areaTimes(flight.areas)?.value ?? '—'
					} satisfies TableColumn<Flight>
				]
			: []),
		{ key: 'pilot_name', header: 'Pilot' },
		{ key: 'aircraft_serial', header: 'Drone serial', identifier: true }
	]);
</script>

{#snippet periodToolbar()}
	<PeriodToolbar
		label="Sessions"
		period={data.state.period}
		timeZone={data.timeZone}
		{at}
		{previousHref}
		{nextHref}
		lines={periodLines}
		open={openSession ? { noun: 'session', earlierHref, laterHref, onclose: close } : null}
	>
		{#snippet tools()}
			<ListFilters
				toolbar
				filters={data.state}
				none={{ ...data.state, ...NO_FILTERS }}
				devices={data.devices}
				groupFilter={readsSeveralGroups(data.account)}
				filtering={cardsFiltering}
				onchange={filter}
			/>
		{/snippet}
	</PeriodToolbar>
{/snippet}

<!-- A session open: its title, the breadcrumb back to the list (through the
     history, so the list is as it was) and no Cards|Table, which has nothing
     to switch there (carbon-expert's advice, user's decision). -->
<svelte:window onkeydown={closeOnEscape} />

<PageHeader
	title={openSession ? `Session ${openedDay}` : 'Sessions'}
	breadcrumb={openSession
		? [{ href: href(data.state), label: 'Sessions', follow: close }]
		: undefined}
	crumb={openSession && openSession.flights[0]
		? `Session ${day(openSession.flights[0].takeoff_at, data.timeZone)}`
		: crumb}
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
		{#if openSession && canEditOpen}
			<!-- Deleting is the way to end a session with comments, which keeps
			     its last flight (user's decision, 2026-10-25). -->
			<Modal
				title="Delete session"
				triggerKind="danger-tertiary"
				submitLabel="Delete session"
				danger
				action="?/deleteSession"
				error={form && 'deleteMessage' in form ? form.deleteMessage : undefined}
				onsuccess={() => toast('Session deleted.')}
			>
				<input type="hidden" name="session_id" value={openSession.id} />
				<p>{deleteConsequences}</p>
			</Modal>
		{/if}
	{/snippet}
</PageHeader>

<Grid>
	<Column narrow>
		{#if data.state.view === 'table' && (data.total > 0 || filtering)}
			<ListFilters
				filters={data.state}
				none={{ ...data.state, ...NO_FILTERS }}
				devices={data.devices}
				groupFilter={readsSeveralGroups(data.account)}
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
			<!-- The cards, and in their place the open session's facts; the toolbar
			     stays when a session opens, only its controls change. The map
			     below lg only for an open session, under its facts. -->
			<CardsView
				opened={opened !== null}
				showMap={wide.current || openSession !== null}
				toolbar={periodToolbar}
			>
				{#snippet cards()}
					{#if data.total === 0}
						<EmptyState
							title="No sessions in {periodName(data.state.period)}"
							body={cardsFiltering
								? 'No sessions you can see in this period have a flight that passes these filters.'
								: previousHref || nextHref
									? 'Step to another period with the arrows, or see all sessions in the table.'
									: 'No sessions yet.'}
						>
							{#snippet action()}
								{#if cardsFiltering}
									<Button kind="ghost" onclick={() => filter(NO_FILTERS)}>Clear filters</Button>
								{:else if previousHref || nextHref}
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
									<li>
										<ClickableTile
											href={sessionHref(session.id)}
											hovered={wide.current && pointedTrack === session.id}
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
												{sessionWho(session)}
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
				{/snippet}
				{#snippet facts()}
					{#if openSession}
						<div class="panel">
							<SessionFacts
								flights={openSession.flights}
								timeZone={data.timeZone}
								group={sessionGroup(openSession)}
								showGroup={showsGroups(data.account)}
								showAreas={usesAreas(data.account)}
							>
								{#snippet pilotAction()}
									{#if canSetPilots}
										<IconButton
											icon={Edit}
											label="Change pilot"
											name="Change pilot of the session’s flights"
											onclick={(event) =>
												changePilot(
													openFlights,
													`All ${flightsLabel(openFlights.length)} in the session.`,
													event
												)}
										/>
									{/if}
								{/snippet}
								{#snippet groupAction()}
									{#if canMoveSession}
										<!-- carbon-expert's advice: the session only by default; moving
											     its flights too only for who may. -->
										<Modal
											title="Move to group"
											triggerIcon={Edit}
											submitLabel="Move"
											action="?/setGroup"
											error={form && 'groupMessage' in form ? form.groupMessage : undefined}
											onsuccess={sessionMoved}
										>
											<input type="hidden" name="session_id" value={openSession?.id} />
											<Select
												label="Group"
												name="group_id"
												options={moveOptions}
												bind:value={moveTo}
											/>
											{#if canMoveFlights && othersThere > 0}
												<Checkbox
													label="Also move its flights to this group"
													name="move_flights"
													value="true"
												/>
												<p class="help">
													{othersThere === 1
														? `1 of its ${flightsLabel(openFlights.length)} is in another group.`
														: `${othersThere} of its ${flightsLabel(openFlights.length)} are in other groups.`}
												</p>
											{/if}
										</Modal>
									{/if}
								{/snippet}
							</SessionFacts>
						</div>
						<!-- Keyed: a draft or an edit never follows to another session. -->
						{#key openSession.id}
							<SessionComments
								sessionId={openSession.id}
								comments={openSession.comments}
								timeZone={data.timeZone}
								userId={data.account?.id ?? ''}
								canWrite={canEditOpen}
								addError={form && 'commentMessage' in form ? form.commentMessage : undefined}
								editError={form && 'editCommentMessage' in form
									? form.editCommentMessage
									: undefined}
								deleteError={form && 'deleteCommentMessage' in form
									? form.deleteCommentMessage
									: undefined}
							/>
						{/key}
					{/if}
				{/snippet}
				{#snippet map()}
					<!-- The tracks aren't focusable: the cards are the same sessions. -->
					<TrackMap
						tracks={mapTracks}
						{highlight}
						ends={openSession !== null}
						areas={openSession ? data.areas : []}
						reserveLegend
						fly
						height={MAP_HEIGHT}
						ontrackclick={opened ? undefined : openTrack}
						ontrackhover={opened
							? (index) => (pointedFlightTrack = index)
							: (index) => (pointedTrack = index === null ? null : (sessionOf(index)?.id ?? null))}
						tracktip={opened ? flightTip : trackTip}
						token={data.mapboxToken}
						label={openSession
							? 'Map of the session’s tracks'
							: `Map of the sessions in ${periodName(data.state.period)}; the list beside it has the same sessions`}
					/>
				{/snippet}
			</CardsView>
			{#if openSession}
				<div class="opened-flights" id="session-flights">
					<DataTable
						label="Flights of the session"
						columns={flightColumns}
						rows={openFlights}
						rowKey={(flight) => flight.id}
						rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
						onrowhover={(flight) => (pointedFlight = flight ? openFlights.indexOf(flight) : null)}
						hoveredRow={pointedFlightTrack === null
							? null
							: (openFlights[pointedFlightTrack]?.id ?? null)}
						selectable={canSetPilots || canEditOpen ? () => true : undefined}
						bind:selected={selectedFlights}
						selectedLabel={(count) => `${flightsLabel(count)} selected`}
						summary={flightsLabel(openFlights.length)}
					>
						{#snippet batchActions(ids)}
							{#if canSetPilots}
								<Button
									onclick={(event: MouseEvent) =>
										changePilot(
											openFlights.filter((flight) => ids.includes(flight.id)),
											`The ${flightsLabel(ids.length)} selected.`,
											event
										)}>Change pilot</Button
								>
							{/if}
							{#if canEditOpen && ids.length < openFlights.length}
								<Modal
									title="Move to new session"
									submitLabel="Move"
									action="?/split"
									error={form?.splitMessage}
									onsuccess={() => moved(ids.length)}
								>
									{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
									<p>
										The {ids.length === 1 ? 'flight becomes' : `${ids.length} flights become`} a session
										of {ids.length === 1 ? 'its' : 'their'} own. Merge sessions in the list to undo this.
									</p>
								</Modal>
							{/if}
							{#if canEditOpen}
								<Modal
									title="Remove from session"
									submitLabel="Remove"
									action="?/remove"
									error={form?.removeMessage}
									onsuccess={() => removed(ids.length)}
								>
									{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
									<input
										type="hidden"
										name="all"
										value={String(ids.length === openFlights.length)}
									/>
									<p>
										{#if ids.length === openFlights.length}
											The session ends. Its {ids.length === 1 ? 'flight stays' : 'flights stay'} in Flights,
											without a session.
										{:else}
											The {ids.length === 1 ? 'flight leaves' : `${ids.length} flights leave`} the session.
											{ids.length === 1 ? 'It stays' : 'They stay'} in Flights, without a session.
										{/if}
									</p>
								</Modal>
							{/if}
						{/snippet}
					</DataTable>
				</div>
			{/if}
		{/if}
		{#if data.state.view === 'table' && data.total > 0}
			<DataTable
				label="Sessions"
				{columns}
				rows={data.sessions}
				rowKey={(session) => session.id}
				rowHref={(session) =>
					`${resolve('/sessions')}${query(
						{
							...data.state,
							view: 'cards',
							period: periodOf('month', dayOf(session.first_takeoff_at, data.timeZone))
						},
						session.id
					)}`}
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

<PilotDialog
	flights={pilotFor}
	which={pilotWhich}
	choices={data.choices}
	bind:open={pilotOpen}
	returnFocus={() => (pilotOpener?.isConnected ? pilotOpener : undefined)}
	error={form?.pilotMessage}
	onsuccess={pilotSet}
/>

<style>
	/* Helper text under the checkbox in Move to group. */
	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}

	/* The flights under the map and facts, coming in after the facts. */
	.opened-flights {
		margin-top: var(--cds-spacing-05);
		animation: 240ms cubic-bezier(0, 0, 0.38, 0.9) 360ms both flights-in;
	}

	@keyframes flights-in {
		from {
			opacity: 0;
			transform: translateY(2rem);
		}
	}
</style>
