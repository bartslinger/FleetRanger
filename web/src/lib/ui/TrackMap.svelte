<!--
	@component
	A Mapbox map of flight tracks, fitted to them, with a marker at each
	takeoff (hollow) and landing (filled). Satellite with streets by default;
	the switch offers a street map in the current theme.
	Mapbox isn't part of Carbon. The switch follows Carbon's content switcher
	(small); the zoom buttons and attribution are Mapbox's own.
-->
<script lang="ts" module>
	/** A GeoJSON LineString: [longitude, latitude, height above takeoff]. */
	export interface Track {
		type: 'LineString';
		coordinates: number[][];
	}
</script>

<script lang="ts">
	import 'mapbox-gl/dist/mapbox-gl.css';
	import type { ExpressionSpecification, Map as MapboxMap, GeoJSONSource } from 'mapbox-gl';
	import { addAreaLayers, bounds, setAreas, type Area } from './areas.js';
	import { Tooltip } from 'bits-ui';
	import InlineNotification from './InlineNotification.svelte';
	import TooltipContent from './TooltipContent.svelte';
	import Loading from './Loading.svelte';

	interface Props {
		tracks: Track[];
		/** The install's public Mapbox token; without one there is no map. */
		token: string;
		/** For screen readers, e.g. "Map of the flight's track". */
		label: string;
		/** Tracks are on their way: shows Carbon's inline loading over the map. */
		loading?: boolean;
		/** The tracks to bring out, by index, e.g. the flight pointed at in a list
		 * or a session's flights. */
		highlight?: number | number[] | null;
		/** The organisation's areas, outlined under the tracks. */
		areas?: Area[];
		/** Takeoff and landing markers; left out on a map of many tracks. */
		ends?: boolean;
		/** A track was clicked, by index. */
		ontrackclick?: (index: number) => void;
		/** The track under the pointer, by index; null when it leaves. */
		ontrackhover?: (index: number | null) => void;
		/** A tooltip for the track under the pointer, e.g. which session. */
		tracktip?: (index: number) => string;
		/** A CSS height, e.g. to fill the window beside a list; at least 20rem. */
		height?: string;
		/** Keep the legend's row also while it has nothing to say, so one
		 * appearing later (e.g. a session's takeoffs and landings) moves
		 * nothing under the map. */
		reserveLegend?: boolean;
		/** Other tracks are flown to, not jumped to: e.g. from a month's
		 * sessions to one of them, and from one session to the next. */
		fly?: boolean;
	}

	let {
		tracks,
		token,
		label,
		loading = false,
		highlight = null,
		areas = [],
		ends: showEnds = true,
		ontrackclick,
		ontrackhover,
		tracktip,
		height,
		reserveLegend = false,
		fly = false
	}: Props = $props();

	let pointedTrack: { index: number; x: number; y: number } | null = $state(null);
	let tipAnchor: HTMLElement | undefined = $state();

	type View = 'satellite' | 'map';
	let view: View = $state('satellite');
	let container: HTMLDivElement | undefined = $state();
	let map: MapboxMap | undefined = $state.raw();
	let failed = $state(false);

	// Map paint can't read CSS variables, so these are Carbon's values:
	// Cyan 50, second in Carbon's categorical palette (user's choice,
	// 2026-10-21, over magenta 50 and teal 50, which faded into green
	// imagery), with a thin Cyan 80 casing on every map (user's choice, over
	// white, Cyan 20 and 30, and Gray 100).
	const TRACK = '#1192e8';
	const CASING = '#003a6d';

	// mapbox-gl doesn't export its GeoJSON types.
	type FeatureCollection = Extract<
		Parameters<GeoJSONSource['setData']>[0],
		{ type: 'FeatureCollection' }
	>;
	type Feature = FeatureCollection['features'][number];

	function dark() {
		const chosen = document.documentElement.dataset.theme;
		if (chosen) return chosen === 'dark';
		return matchMedia('(prefers-color-scheme: dark)').matches;
	}

	function styleUrl(view: View) {
		if (view === 'satellite') return 'mapbox://styles/mapbox/satellite-streets-v12';
		return dark() ? 'mapbox://styles/mapbox/dark-v11' : 'mapbox://styles/mapbox/streets-v12';
	}

	function lines(tracks: Track[]): FeatureCollection {
		return {
			type: 'FeatureCollection',
			features: tracks
				.map((track, index) => ({
					type: 'Feature' as const,
					properties: { index },
					geometry: track
				}))
				.filter((feature) => feature.geometry.coordinates.length > 1)
		};
	}

	function ends(tracks: Track[]): FeatureCollection {
		return {
			type: 'FeatureCollection',
			// With their track's index, to step back with it.
			features: (showEnds ? tracks : []).flatMap((track, index) =>
				track.coordinates.length > 0
					? [
							point(track.coordinates[0], 'takeoff', index),
							point(track.coordinates[track.coordinates.length - 1], 'landing', index)
						]
					: []
			)
		};
	}

	function point(position: number[], kind: string, index: number): Feature {
		return {
			type: 'Feature',
			properties: { kind, index },
			geometry: { type: 'Point', coordinates: position.slice(0, 2) }
		};
	}

	/** Adds the tracks to the current style; again after every style change. */
	function addLayers(map: MapboxMap) {
		// Areas first, so the tracks are drawn over them.
		// Thin and neutral, behind the tracks: Gray 60 on the light street
		// map, Gray 20 on imagery and the dark map.
		const lightMap = view === 'map' && !dark();
		addAreaLayers(map, areas, null, lightMap ? '#6f6f6f' : '#e0e0e0');
		map.addSource('tracks', { type: 'geojson', data: lines(tracks) });
		map.addSource('ends', { type: 'geojson', data: ends(tracks) });
		map.addLayer({
			id: 'track-casing',
			type: 'line',
			source: 'tracks',
			layout: { 'line-join': 'round', 'line-cap': 'round' },
			paint: { 'line-color': CASING, 'line-width': 5, 'line-opacity': 0.9 }
		});
		map.addLayer({
			id: 'track',
			type: 'line',
			source: 'tracks',
			layout: { 'line-join': 'round', 'line-cap': 'round' },
			paint: { 'line-color': TRACK, 'line-width': 3 }
		});
		// The highlighted track: wider, over the others.
		map.addLayer({
			id: 'highlight-casing',
			type: 'line',
			source: 'tracks',
			filter: highlightFilter(highlight),
			layout: { 'line-join': 'round', 'line-cap': 'round' },
			paint: { 'line-color': CASING, 'line-width': 9 }
		});
		map.addLayer({
			id: 'highlight',
			type: 'line',
			source: 'tracks',
			filter: highlightFilter(highlight),
			layout: { 'line-join': 'round', 'line-cap': 'round' },
			paint: { 'line-color': TRACK, 'line-width': 5 }
		});
		// What the pointer finds a track by: 16px wide, unseen, as the line
		// itself is a thin target.
		map.addLayer({
			id: 'track-hit',
			type: 'line',
			source: 'tracks',
			layout: { 'line-join': 'round', 'line-cap': 'round' },
			paint: { 'line-color': TRACK, 'line-width': 16, 'line-opacity': 0 }
		});
		// A landing: an X drawn as the lines are (an image, as Mapbox draws
		// none), under the takeoffs where a flight lands and the next takes off.
		if (!map.hasImage('landing')) {
			map.addImage('landing', landingImage(3), { pixelRatio: 2 });
			map.addImage('landing-highlight', landingImage(5), { pixelRatio: 2 });
		}
		map.addLayer({
			id: 'landings',
			type: 'symbol',
			source: 'ends',
			filter: ['==', ['get', 'kind'], 'landing'],
			layout: { 'icon-allow-overlap': true, 'icon-ignore-placement': true }
		});
		// A takeoff: a 12px dot with a 1px casing, as thin as the lines'.
		map.addLayer({
			id: 'takeoffs',
			type: 'circle',
			source: 'ends',
			filter: ['==', ['get', 'kind'], 'takeoff'],
			paint: { 'circle-color': TRACK, 'circle-stroke-color': CASING }
		});
		styleEnds(map, highlightFilter(highlight));
	}

	/** The takeoffs and landings of the highlighted tracks wider, as their
	 * lines, and on top; the others' step back with theirs. */
	function styleEnds(map: MapboxMap, filter: ReturnType<typeof highlightFilter>) {
		const dimmed = filter[2][1].length > 0;
		const opacity: ExpressionSpecification | number = dimmed ? ['case', filter, 1, 0.3] : 1;
		map.setPaintProperty('takeoffs', 'circle-radius', ['case', filter, 6, 5]);
		map.setPaintProperty('takeoffs', 'circle-stroke-width', ['case', filter, 2, 1]);
		map.setPaintProperty('takeoffs', 'circle-opacity', opacity);
		map.setPaintProperty('takeoffs', 'circle-stroke-opacity', opacity);
		map.setLayoutProperty('takeoffs', 'circle-sort-key', ['case', filter, 1, 0]);
		map.setLayoutProperty('landings', 'icon-image', [
			'case',
			filter,
			'landing-highlight',
			'landing'
		]);
		map.setPaintProperty('landings', 'icon-opacity', opacity);
		map.setLayoutProperty('landings', 'symbol-sort-key', ['case', filter, 1, 0]);
	}

	// The landing's X in an 18px box, its arms from corner to corner of the
	// middle 8px: the legend draws the same.
	const CROSS = [
		[5, 5, 13, 13],
		[13, 5, 5, 13]
	];

	/** The landing's X at twice the size, for sharp edges: as the line, of
	 * this width over its casing 2px wider on each side (1px when 3px). */
	function landingImage(width: 3 | 5): ImageData {
		const canvas = document.createElement('canvas');
		canvas.width = 36;
		canvas.height = 36;
		const context = canvas.getContext('2d')!;
		context.scale(2, 2);
		context.lineCap = 'round';
		for (const [color, stroke] of [
			[CASING, width === 3 ? 5 : 9],
			[TRACK, width]
		] as const) {
			context.strokeStyle = color;
			context.lineWidth = stroke;
			context.beginPath();
			for (const [x1, y1, x2, y2] of CROSS) {
				context.moveTo(x1, y1);
				context.lineTo(x2, y2);
			}
			context.stroke();
		}
		return context.getImageData(0, 0, canvas.width, canvas.height);
	}

	/** Only the tracks with these indexes; none without. */
	function highlightFilter(
		index: number | number[] | null
	): ['in', ['get', string], ['literal', number[]]] {
		const indexes = index === null ? [] : Array.isArray(index) ? index : [index];
		return ['in', ['get', 'index'], ['literal', indexes]];
	}

	const FIT = { padding: 48, maxZoom: 17 };
	/** A flight to other tracks: within carbon-expert's 600–800ms. */
	const FLY_MS = 700;

	/** The tracks' extent; null without positions. */
	const extentOf = (tracks: Track[]) => bounds(tracks.flatMap((track) => track.coordinates));

	/** Puts the tracks into the sources and fits them. Before `style.load` there
	 * are no sources yet; it adds them with the tracks of that moment. */
	function draw(map: MapboxMap, tracks: Track[]) {
		(map.getSource('tracks') as GeoJSONSource | undefined)?.setData(lines(tracks));
		(map.getSource('ends') as GeoJSONSource | undefined)?.setData(ends(tracks));
		// The first view at once; later ones flown to, where asked.
		fit(map, tracks, fly && placed);
	}

	function fit(map: MapboxMap, tracks: Track[], animate: boolean) {
		const extent = extentOf(tracks);
		if (!extent) {
			// No tracks (e.g. a period without sessions): shown as it is.
			if (!placed) map.once('idle', () => (placed = true));
			return;
		}
		map.fitBounds(extent, { ...FIT, animate, duration: animate ? FLY_MS : 0 });
		// Shown once the new view is drawn: until its tiles are in, Mapbox
		// draws the ones it has, from the starting view (the whole world).
		if (!placed) map.once('idle', () => (placed = true));
	}

	// Hidden until the map shows the tracks, so it never shows another view
	// (the starting view, or the style's) first.
	let placed = $state(false);

	// Only a new token makes a new map: read through the caller's data (e.g.
	// a page's `data.mapboxToken`), the prop changes with every navigation,
	// which made a new map each time, so it never flew (user's feedback).
	const mapToken = $derived(token);

	// Creates the map in the browser; mapbox-gl can't run on the server.
	$effect(() => {
		if (!mapToken || !container) return;
		const element = container;
		let cancelled = false;
		let created: MapboxMap | undefined;
		let resizes: ResizeObserver | undefined;
		import('mapbox-gl').then(({ default: mapboxgl }) => {
			if (cancelled) return;
			// With tracks known, the very first frame already shows them.
			const initial = extentOf(tracks);
			created = new mapboxgl.Map({
				container: element,
				accessToken: mapToken,
				style: styleUrl(view),
				attributionControl: false,
				cooperativeGestures: matchMedia('(pointer: coarse)').matches,
				// An explicit camera: without one, Mapbox jumps to the style's own
				// view (Dubai) when the style loads, also after the tracks were
				// fitted. The bounds then set the actual view.
				center: [0, 0],
				zoom: 1,
				...(initial ? { bounds: initial, fitBoundsOptions: FIT } : {})
			});
			// Shown once `load` below has fitted the tracks.
			created.addControl(new mapboxgl.NavigationControl({ showCompass: false }), 'top-right');
			created.addControl(new mapboxgl.ScaleControl({ unit: 'metric' }), 'bottom-left');
			created.addControl(new mapboxgl.AttributionControl({ compact: true }), 'bottom-right');
			const instance = created;
			instance.on('style.load', () => {
				addLayers(instance);
			});
			instance.on('error', (event) => {
				// A rejected token or an unreachable Mapbox; tile hiccups don't count.
				const status = (event.error as { status?: number } | undefined)?.status;
				if (status === 401 || status === 403 || !instance.isStyleLoaded()) failed = true;
			});
			// Tracks that arrived while the style was loading weren't drawn yet.
			instance.once('load', () => draw(instance, tracks));
			// Mapbox follows only the window's size; this follows the container,
			// e.g. while a layout around it animates.
			resizes = new ResizeObserver(() => instance.resize());
			resizes.observe(element);
			// A track clicked, where the page wants it.
			instance.on('click', 'track-hit', (event) => {
				// mapbox-gl's feature type leaves out the GeoJSON properties.
				const feature = event.features?.[0] as { properties?: { index?: number } } | undefined;
				const index = feature?.properties?.index;
				if (index !== undefined) ontrackclick?.(index);
			});
			instance.on('mouseenter', 'track-hit', () => {
				if (ontrackclick) instance.getCanvas().style.cursor = 'pointer';
			});
			// The track under the pointer, its tooltip following the pointer, as
			// Carbon's charts' do.
			instance.on('mousemove', 'track-hit', (event) => {
				const feature = event.features?.[0] as { properties?: { index?: number } } | undefined;
				const index = feature?.properties?.index;
				if (index === undefined) return;
				const another = index !== pointedTrack?.index;
				pointedTrack = { index, x: event.point.x, y: event.point.y };
				if (another) ontrackhover?.(index);
			});
			instance.on('mouseleave', 'track-hit', () => {
				instance.getCanvas().style.cursor = '';
				pointedTrack = null;
				ontrackhover?.(null);
			});
			map = instance;
		});
		return () => {
			cancelled = true;
			resizes?.disconnect();
			created?.remove();
			map = undefined;
		};
	});

	// New tracks (another flight or session): new data and view. Not gated on
	// isStyleLoaded(): that stays false a while after `style.load` added the
	// sources, and tracks arriving then were never drawn.
	$effect(() => {
		const current = tracks;
		if (!map?.getSource('tracks')) return;
		draw(map, current);
	});

	// Other areas, e.g. after navigating: only the data changes, not the view.
	$effect(() => {
		const current = areas;
		if (map?.getSource('areas')) setAreas(map, current);
	});

	// Another track pointed at: only the filter changes, not the view.
	$effect(() => {
		const filter = highlightFilter(highlight);
		if (!map?.getLayer('highlight')) return;
		map.setFilter('highlight-casing', filter);
		map.setFilter('highlight', filter);
		// The others step back to 30% while some are brought out (Carbon's
		// legend hover), their takeoffs and landings with them.
		const dimmed = filter[2][1].length > 0;
		map.setPaintProperty('track', 'line-opacity', dimmed ? 0.3 : 1);
		map.setPaintProperty('track-casing', 'line-opacity', dimmed ? 0.3 : 0.9);
		styleEnds(map, filter);
	});

	function show(next: View) {
		if (next === view) return;
		view = next;
		map?.setStyle(styleUrl(next));
	}
</script>

{#if !token}
	<InlineNotification kind="info" title="No map">
		This install has no Mapbox token. An administrator can set MAPBOX_ACCESS_TOKEN.
	</InlineNotification>
{:else if failed}
	<InlineNotification kind="error" title="The map couldn’t load">
		Mapbox didn’t answer or refused the token. Try again later.
	</InlineNotification>
{:else}
	<figure class="track-map" style:--track={TRACK} style:--casing={CASING}>
		<div
			class="map"
			class:placed
			style:height={height ? `max(20rem, ${height})` : undefined}
			bind:this={container}
			role="region"
			aria-label={label}
			aria-busy={loading}
		></div>
		<!-- For the mouse only, and nothing only it says: e.g. the list beside
		     the map has the same. Carbon's default tooltip, above the point. -->
		{#if tracktip}
			<Tooltip.Root open={pointedTrack !== null}>
				<span
					class="tip-anchor"
					style:left="{pointedTrack?.x ?? 0}px"
					style:top="{pointedTrack?.y ?? 0}px"
					bind:this={tipAnchor}
				></span>
				{#if pointedTrack}
					<TooltipContent side="top" anchor={tipAnchor} follow
						>{tracktip(pointedTrack.index)}</TooltipContent
					>
				{/if}
			</Tooltip.Root>
		{/if}
		{#if loading}
			<p class="loading" role="status"><Loading /> Loading tracks…</p>
		{/if}
		<div class="switcher" role="group" aria-label="Map style">
			{#each [['satellite', 'Satellite'], ['map', 'Map']] as const as [value, text] (value)}
				<button
					type="button"
					class:selected={view === value}
					aria-pressed={view === value}
					onclick={() => show(value)}>{text}</button
				>
			{/each}
		</div>
		{#if showEnds || areas.length > 0 || reserveLegend}
			<figcaption class="legend" class:reserved={reserveLegend}>
				{#if showEnds}
					<span><span class="marker takeoff" aria-hidden="true"></span>Takeoff</span>
					<span
						><svg class="landing" viewBox="2 2 14 14" aria-hidden="true">
							{#each CROSS as [x1, y1, x2, y2], arm (arm)}<line
									{x1}
									{y1}
									{x2}
									{y2}
									class="casing"
								/>{/each}
							{#each CROSS as [x1, y1, x2, y2], arm (arm)}<line {x1} {y1} {x2} {y2} />{/each}
						</svg>Landing</span
					>
				{/if}
				{#if areas.length > 0}
					<span><span class="marker area" aria-hidden="true"></span>Area</span>
				{/if}
			</figcaption>
		{/if}
	</figure>
{/if}

<style>
	.track-map {
		position: relative;
		margin: 0;
	}

	.tip-anchor {
		position: absolute;
		width: 0;
		height: 0;
		pointer-events: none;
	}

	.map {
		height: clamp(20rem, 60vh, 40rem);
		background: var(--cds-layer-01);
	}

	/* Mapbox draws into the canvas; until the tracks are fitted, don't show it. */
	.map:not(.placed) :global(canvas) {
		visibility: hidden;
	}

	/* Carbon's inline loading, on a layer over the map's centre. */
	.loading {
		position: absolute;
		top: 50%;
		left: 50%;
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
		margin: 0;
		padding: var(--cds-spacing-03) var(--cds-spacing-05);
		transform: translate(-50%, -50%);
		background: var(--cds-layer-01);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* Carbon's content switcher, small (32px), over the map's top left. */
	.switcher {
		position: absolute;
		top: var(--cds-spacing-03);
		left: var(--cds-spacing-03);
		display: flex;
	}

	.switcher button {
		height: 2rem;
		padding: 0 var(--cds-spacing-05);
		border: 1px solid var(--cds-border-inverse);
		background: var(--cds-layer-01);
		color: var(--cds-text-secondary);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
		transition: background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.switcher button + button {
		border-left: 0;
	}

	.switcher button:hover:not(.selected) {
		background: var(--cds-layer-hover-01);
		color: var(--cds-text-primary);
	}

	.switcher button.selected {
		background: var(--cds-layer-selected-inverse);
		color: var(--cds-text-inverse);
		cursor: default;
	}

	.switcher button:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
		box-shadow: inset 0 0 0 3px var(--cds-focus-inset);
	}

	/* As high empty as with entries: label-01's 16px line. */
	.legend.reserved {
		min-height: var(--cds-spacing-05);
	}

	.legend {
		display: flex;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-03);
		/* The map hangs into the gutter (narrow mode); the text lines up with the page's. */
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.legend > span {
		display: inline-flex;
		align-items: center;
		gap: var(--cds-spacing-02);
	}

	/* The map's markers, at the same size. */
	.marker {
		box-sizing: border-box;
		width: 0.75rem;
		height: 0.75rem;
	}

	.takeoff {
		border: 1px solid var(--casing);
		border-radius: 50%;
		background: var(--track);
	}

	.landing {
		width: 0.875rem;
		height: 0.875rem;
	}

	.landing line {
		stroke: var(--track);
		stroke-width: 3;
		stroke-linecap: round;
	}

	.landing .casing {
		stroke: var(--casing);
		stroke-width: 5;
	}

	/* An area: a thin neutral border, as on the map (Gray 50, between the
	   map's two greys). Literal like the markers': the map's paint can't
	   read tokens. */
	.area {
		border: 1px solid #8d8d8d;
	}

	@media (prefers-reduced-motion: reduce) {
		.switcher button {
			transition: none;
		}
	}
</style>
