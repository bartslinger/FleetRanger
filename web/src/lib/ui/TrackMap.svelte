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
	import type { Map as MapboxMap, GeoJSONSource } from 'mapbox-gl';
	import { addAreaLayers, bounds, setAreas, type Area } from './areas.js';
	import InlineNotification from './InlineNotification.svelte';
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
		/** A CSS height, e.g. to fill the window beside a list; at least 20rem. */
		height?: string;
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
		height
	}: Props = $props();

	type View = 'satellite' | 'map';
	let view: View = $state('satellite');
	let container: HTMLDivElement | undefined = $state();
	let map: MapboxMap | undefined = $state.raw();
	let failed = $state(false);

	// Map paint can't read CSS variables, so these are Carbon's values:
	// Magenta 50 stands out on imagery and on both street maps, with a white
	// casing (Gray 100 on the dark map).
	const TRACK = '#ee5396';
	const CASING = { light: '#ffffff', dark: '#161616' };

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
			features: (showEnds ? tracks : [])
				.filter((track) => track.coordinates.length > 0)
				.flatMap((track) => [
					point(track.coordinates[0], 'takeoff'),
					point(track.coordinates[track.coordinates.length - 1], 'landing')
				])
		};
	}

	function point(position: number[], kind: string): Feature {
		return {
			type: 'Feature',
			properties: { kind },
			geometry: { type: 'Point', coordinates: position.slice(0, 2) }
		};
	}

	/** Adds the tracks to the current style; again after every style change. */
	function addLayers(map: MapboxMap, casing: string) {
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
			paint: { 'line-color': casing, 'line-width': 5, 'line-opacity': 0.9 }
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
			paint: { 'line-color': casing, 'line-width': 9 }
		});
		map.addLayer({
			id: 'highlight',
			type: 'line',
			source: 'tracks',
			filter: highlightFilter(highlight),
			layout: { 'line-join': 'round', 'line-cap': 'round' },
			paint: { 'line-color': TRACK, 'line-width': 5 }
		});
		map.addLayer({
			id: 'ends',
			type: 'circle',
			source: 'ends',
			paint: {
				'circle-radius': 6,
				'circle-color': ['match', ['get', 'kind'], 'takeoff', casing, TRACK],
				'circle-stroke-color': ['match', ['get', 'kind'], 'takeoff', TRACK, casing],
				'circle-stroke-width': 3
			}
		});
	}

	/** Only the tracks with these indexes; none without. */
	function highlightFilter(
		index: number | number[] | null
	): ['in', ['get', string], ['literal', number[]]] {
		const indexes = index === null ? [] : Array.isArray(index) ? index : [index];
		return ['in', ['get', 'index'], ['literal', indexes]];
	}

	const FIT = { padding: 48, maxZoom: 17 };

	/** The tracks' extent; null without positions. */
	const extentOf = (tracks: Track[]) => bounds(tracks.flatMap((track) => track.coordinates));

	/** Puts the tracks into the sources and fits them. Before `style.load` there
	 * are no sources yet; it adds them with the tracks of that moment. */
	function draw(map: MapboxMap, tracks: Track[]) {
		(map.getSource('tracks') as GeoJSONSource | undefined)?.setData(lines(tracks));
		(map.getSource('ends') as GeoJSONSource | undefined)?.setData(ends(tracks));
		fit(map, tracks, false);
	}

	function fit(map: MapboxMap, tracks: Track[], animate: boolean) {
		const extent = extentOf(tracks);
		if (!extent) return;
		map.fitBounds(extent, { ...FIT, animate });
		// Shown once the new view is drawn: until its tiles are in, Mapbox
		// draws the ones it has, from the starting view (the whole world).
		if (!placed) map.once('idle', () => (placed = true));
	}

	// Hidden until the map shows the tracks, so it never shows another view
	// (the starting view, or the style's) first.
	let placed = $state(false);

	// Creates the map in the browser; mapbox-gl can't run on the server.
	$effect(() => {
		if (!token || !container) return;
		const element = container;
		let cancelled = false;
		let created: MapboxMap | undefined;
		import('mapbox-gl').then(({ default: mapboxgl }) => {
			if (cancelled) return;
			// With tracks known, the very first frame already shows them.
			const initial = extentOf(tracks);
			created = new mapboxgl.Map({
				container: element,
				accessToken: token,
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
				const darkMap = view === 'map' && dark();
				addLayers(instance, darkMap ? CASING.dark : CASING.light);
			});
			instance.on('error', (event) => {
				// A rejected token or an unreachable Mapbox; tile hiccups don't count.
				const status = (event.error as { status?: number } | undefined)?.status;
				if (status === 401 || status === 403 || !instance.isStyleLoaded()) failed = true;
			});
			// Tracks that arrived while the style was loading weren't drawn yet.
			instance.once('load', () => draw(instance, tracks));
			// A track clicked, where the page wants it.
			instance.on('click', 'track', (event) => {
				// mapbox-gl's feature type leaves out the GeoJSON properties.
				const feature = event.features?.[0] as { properties?: { index?: number } } | undefined;
				const index = feature?.properties?.index;
				if (index !== undefined) ontrackclick?.(index);
			});
			instance.on('mouseenter', 'track', () => {
				if (ontrackclick) instance.getCanvas().style.cursor = 'pointer';
			});
			instance.on('mouseleave', 'track', () => (instance.getCanvas().style.cursor = ''));
			map = instance;
		});
		return () => {
			cancelled = true;
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
		// The others step back while some are brought out.
		const dimmed = filter[2][1].length > 0;
		map.setPaintProperty('track', 'line-opacity', dimmed ? 0.35 : 1);
		map.setPaintProperty('track-casing', 'line-opacity', dimmed ? 0.3 : 0.9);
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
	<figure class="track-map">
		<div
			class="map"
			class:placed
			style:height={height ? `max(20rem, ${height})` : undefined}
			bind:this={container}
			role="region"
			aria-label={label}
			aria-busy={loading}
		></div>
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
		{#if showEnds || areas.length > 0}
			<figcaption class="legend">
				{#if showEnds}
					<span><span class="marker takeoff" aria-hidden="true"></span>Takeoff</span>
					<span><span class="marker landing" aria-hidden="true"></span>Landing</span>
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
		border: 3px solid #ee5396;
		border-radius: 50%;
	}

	.takeoff {
		background: #ffffff;
	}

	/* An area: a thin neutral border, as on the map (Gray 50, between the
	   map's two greys). Literal like the markers': the map's paint can't
	   read tokens. */
	.area {
		border: 1px solid #8d8d8d;
		border-radius: 0;
	}

	.landing {
		background: #ee5396;
	}

	@media (prefers-reduced-motion: reduce) {
		.switcher button {
			transition: none;
		}
	}
</style>
