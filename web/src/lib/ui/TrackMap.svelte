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
		/** Square below md, so on a phone the content under the map stays in view. */
		squareOnPhones?: boolean;
	}

	let { tracks, token, label, loading = false, squareOnPhones = false }: Props = $props();

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
				.filter((track) => track.coordinates.length > 1)
				.map((track) => ({ type: 'Feature', properties: {}, geometry: track }))
		};
	}

	function ends(tracks: Track[]): FeatureCollection {
		return {
			type: 'FeatureCollection',
			features: tracks
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

	const FIT = { padding: 48, maxZoom: 17 };

	/** The tracks' extent as [[west, south], [east, north]]; null without positions. */
	function bounds(tracks: Track[]): [[number, number], [number, number]] | null {
		const positions = tracks.flatMap((track) => track.coordinates);
		if (positions.length === 0) return null;
		let [west, south, east, north] = [180, 90, -180, -90];
		for (const [lon, lat] of positions) {
			west = Math.min(west, lon);
			east = Math.max(east, lon);
			south = Math.min(south, lat);
			north = Math.max(north, lat);
		}
		return [
			[west, south],
			[east, north]
		];
	}

	function fit(map: MapboxMap, tracks: Track[], animate: boolean) {
		const extent = bounds(tracks);
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
			const initial = bounds(tracks);
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
			// Tracks that arrived while the style was loading weren't fitted yet.
			instance.once('load', () => fit(instance, tracks, false));
			map = instance;
		});
		return () => {
			cancelled = true;
			created?.remove();
			map = undefined;
		};
	});

	// Another flight on the same page component: new data and view.
	$effect(() => {
		const current = tracks;
		if (!map?.isStyleLoaded()) return;
		(map.getSource('tracks') as GeoJSONSource | undefined)?.setData(lines(current));
		(map.getSource('ends') as GeoJSONSource | undefined)?.setData(ends(current));
		fit(map, current, false);
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
			class:square={squareOnPhones}
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
		<figcaption class="legend">
			<span><span class="marker takeoff" aria-hidden="true"></span>Takeoff</span>
			<span><span class="marker landing" aria-hidden="true"></span>Landing</span>
		</figcaption>
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

	@media (max-width: 41.99rem) {
		.map.square {
			height: auto;
			aspect-ratio: 1;
		}
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

	.landing {
		background: #ee5396;
	}

	@media (prefers-reduced-motion: reduce) {
		.switcher button {
			transition: none;
		}
	}
</style>
