<!--
	@component
	A Mapbox map of the organisation's areas, for managing them: every area as
	an outline with its name, and the one being drawn or edited in Terra Draw.
	Drawing snaps to the corners and edges of the other areas, so neighbours
	can share a border. Mapbox and Terra Draw aren't part of Carbon; the
	colours are Carbon's.
-->
<script lang="ts" module>
	/** An area as `GET /api/areas` gives it. */
	export interface Area {
		id: string;
		name: string;
		/** GeoJSON Polygon or MultiPolygon, [longitude, latitude]. */
		boundary: Boundary;
	}

	export type Boundary =
		| { type: 'Polygon'; coordinates: number[][][] }
		| { type: 'MultiPolygon'; coordinates: number[][][][] };

	/** What can be done while editing, for the buttons beside the map. */
	export interface EditState {
		/** Finished parts. */
		parts: number;
		/** A part is selected, so it can be removed. */
		partSelected: boolean;
		/** A part is being drawn. */
		drawing: boolean;
	}
</script>

<script lang="ts">
	import 'mapbox-gl/dist/mapbox-gl.css';
	import type { Map as MapboxMap, GeoJSONSource } from 'mapbox-gl';
	import type { TerraDraw, TerraDrawMouseEvent } from 'terra-draw';
	import InlineNotification from './InlineNotification.svelte';

	interface Props {
		areas: Area[];
		/** The install's public Mapbox token; without one there is no map. */
		token: string;
		/** For screen readers, e.g. "Map of the areas". */
		label: string;
		/** The area to bring out. */
		selected?: string | null;
		/** The area being edited, `new` for one being added, null for none. */
		editing?: string | null;
		/** Where to look when there are no areas yet, e.g. a recent flight's track. */
		fallback?: number[][];
		/** An area was clicked on the map (not while editing). */
		onselect?: (id: string) => void;
		onedit?: (state: EditState) => void;
	}

	let {
		areas,
		token,
		label,
		selected = null,
		editing = null,
		fallback = [],
		onselect,
		onedit
	}: Props = $props();

	let container: HTMLDivElement | undefined = $state();
	let map: MapboxMap | undefined = $state.raw();
	let draw: TerraDraw | undefined = $state.raw();
	let failed = $state(false);

	// Map paint can't read CSS variables, so these are Carbon's values: Blue 50
	// outlines with a white casing and a light Blue 60 fill, and labels in
	// white with a Gray 100 halo, readable on satellite imagery.
	const OUTLINE = '#4589ff';
	const FILL = '#0f62fe';
	const CASING = '#ffffff';
	const HALO = '#161616';
	// Terra Draw's handles: white with a Blue 60 outline.
	const HANDLE = '#ffffff';
	/** How close (in pixels) a corner must come to another area to snap to it. */
	const SNAP_PX = 12;

	// mapbox-gl doesn't export its GeoJSON types.
	type FeatureCollection = Extract<
		Parameters<GeoJSONSource['setData']>[0],
		{ type: 'FeatureCollection' }
	>;

	/** A boundary's parts, each as its rings. */
	function parts(boundary: Boundary): number[][][][] {
		return boundary.type === 'Polygon' ? [boundary.coordinates] : boundary.coordinates;
	}

	/** The areas shown as Mapbox layers: all but the one being edited. */
	function shown(areas: Area[], editing: string | null, selected: string | null) {
		return {
			type: 'FeatureCollection',
			features: areas
				.filter((area) => area.id !== editing)
				.map((area) => ({
					type: 'Feature' as const,
					properties: { id: area.id, name: area.name, selected: area.id === selected },
					geometry: area.boundary
				}))
		} as FeatureCollection;
	}

	/** One point per area for its name: Mapbox would label a polygon once per tile. */
	function labels(areas: Area[], editing: string | null) {
		return {
			type: 'FeatureCollection',
			features: areas
				.filter((area) => area.id !== editing)
				.map((area) => ({
					type: 'Feature' as const,
					properties: { name: area.name },
					geometry: { type: 'Point' as const, coordinates: labelPoint(area.boundary) }
				}))
		} as FeatureCollection;
	}

	/** A point inside the largest part: the middle of the widest stretch of a
	 * line across it, halfway between its north and south. */
	function labelPoint(boundary: Boundary): number[] {
		const extent = (ring: number[][]) =>
			bounds(ring) ?? [
				[0, 0],
				[0, 0]
			];
		const size = (ring: number[][]) => {
			const [[west, south], [east, north]] = extent(ring);
			return (east - west) * (north - south);
		};
		const rings = parts(boundary).reduce((a, b) => (size(b[0]) > size(a[0]) ? b : a));
		const [[west, south], [east, north]] = extent(rings[0]);
		const y = (south + north) / 2;
		const crossings: number[] = [];
		for (const ring of rings) {
			for (let i = 0; i + 1 < ring.length; i++) {
				const [[x1, y1], [x2, y2]] = [ring[i], ring[i + 1]];
				if (y1 <= y !== y2 <= y) crossings.push(x1 + ((y - y1) * (x2 - x1)) / (y2 - y1));
			}
		}
		crossings.sort((a, b) => a - b);
		let best = [(west + east) / 2, y];
		let widest = -1;
		for (let i = 0; i + 1 < crossings.length; i += 2) {
			if (crossings[i + 1] - crossings[i] > widest) {
				widest = crossings[i + 1] - crossings[i];
				best = [(crossings[i] + crossings[i + 1]) / 2, y];
			}
		}
		return best;
	}

	function addLayers(map: MapboxMap) {
		map.addSource('areas', { type: 'geojson', data: shown(areas, editing, selected) });
		map.addLayer({
			id: 'area-fill',
			type: 'fill',
			source: 'areas',
			paint: {
				'fill-color': FILL,
				'fill-opacity': ['case', ['get', 'selected'], 0.35, 0.15]
			}
		});
		map.addLayer({
			id: 'area-casing',
			type: 'line',
			source: 'areas',
			paint: {
				'line-color': CASING,
				'line-width': ['case', ['get', 'selected'], 6, 4],
				'line-opacity': 0.8
			}
		});
		map.addLayer({
			id: 'area-outline',
			type: 'line',
			source: 'areas',
			paint: { 'line-color': OUTLINE, 'line-width': ['case', ['get', 'selected'], 4, 2] }
		});
		map.addSource('area-labels', { type: 'geojson', data: labels(areas, editing) });
		map.addLayer({
			id: 'area-label',
			type: 'symbol',
			source: 'area-labels',
			layout: {
				'text-field': ['get', 'name'],
				'text-font': ['DIN Pro Medium', 'Arial Unicode MS Regular'],
				'text-size': 14
			},
			paint: { 'text-color': '#ffffff', 'text-halo-color': HALO, 'text-halo-width': 1.5 }
		});
	}

	/** The extent of these positions as [[west, south], [east, north]]. */
	function bounds(positions: number[][]): [[number, number], [number, number]] | null {
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

	function corners(areas: Area[]): number[][] {
		return areas.flatMap((area) => parts(area.boundary).flatMap((rings) => rings[0]));
	}

	const FIT = { padding: 48, maxZoom: 17 };

	function reducedMotion() {
		return matchMedia('(prefers-reduced-motion: reduce)').matches;
	}

	/** Snaps a corner to the nearest corner of another area, or else to the
	 * nearest point on one of its edges, within `SNAP_PX`. */
	function snapToNeighbours(
		event: TerraDrawMouseEvent,
		context: {
			project: (lng: number, lat: number) => { x: number; y: number };
			unproject: (x: number, y: number) => { lng: number; lat: number };
		}
	): number[] | undefined {
		const pointer = { x: event.containerX, y: event.containerY };
		const rings = areas
			.filter((area) => area.id !== editing)
			.flatMap((area) => parts(area.boundary).flat());
		let best: number[] | undefined;
		let nearest = SNAP_PX;
		for (const ring of rings) {
			for (const corner of ring) {
				const at = context.project(corner[0], corner[1]);
				const distance = Math.hypot(at.x - pointer.x, at.y - pointer.y);
				if (distance < nearest) [best, nearest] = [corner, distance];
			}
		}
		if (best) return best;
		for (const ring of rings) {
			for (let i = 0; i + 1 < ring.length; i++) {
				const a = context.project(ring[i][0], ring[i][1]);
				const b = context.project(ring[i + 1][0], ring[i + 1][1]);
				const length2 = (b.x - a.x) ** 2 + (b.y - a.y) ** 2;
				if (length2 === 0) continue;
				const t = Math.max(
					0,
					Math.min(1, ((pointer.x - a.x) * (b.x - a.x) + (pointer.y - a.y) * (b.y - a.y)) / length2)
				);
				const x = a.x + t * (b.x - a.x);
				const y = a.y + t * (b.y - a.y);
				const distance = Math.hypot(x - pointer.x, y - pointer.y);
				if (distance < nearest) {
					const { lng, lat } = context.unproject(x, y);
					[best, nearest] = [[lng, lat], distance];
				}
			}
		}
		return best;
	}

	/** Tells the page what can be done now. */
	function report() {
		if (!draw) return;
		const polygons = draw
			.getSnapshot()
			.filter(
				(feature) => feature.geometry.type === 'Polygon' && feature.properties.mode === 'polygon'
			);
		const drawing =
			draw.getMode() === 'polygon' && polygons.some((f) => f.properties.currentlyDrawing);
		onedit?.({
			parts: polygons.filter((f) => !f.properties.currentlyDrawing).length,
			partSelected: polygons.some((f) => f.properties.selected),
			drawing
		});
	}

	async function startDrawing(instance: MapboxMap) {
		const [{ TerraDraw, TerraDrawPolygonMode, TerraDrawSelectMode }, { TerraDrawMapboxGLAdapter }] =
			await Promise.all([import('terra-draw'), import('terra-draw-mapbox-gl-adapter')]);
		const snapping = { toCoordinate: true, toCustom: snapToNeighbours };
		const created = new TerraDraw({
			adapter: new TerraDrawMapboxGLAdapter({ map: instance }),
			modes: [
				new TerraDrawPolygonMode({
					snapping,
					styles: {
						fillColor: FILL,
						fillOpacity: 0.3,
						outlineColor: OUTLINE,
						outlineWidth: 3,
						closingPointColor: HANDLE,
						closingPointOutlineColor: FILL,
						closingPointWidth: 6,
						closingPointOutlineWidth: 2,
						snappingPointColor: HANDLE,
						snappingPointOutlineColor: FILL,
						snappingPointWidth: 6,
						snappingPointOutlineWidth: 2
					}
				}),
				new TerraDrawSelectMode({
					flags: {
						polygon: {
							feature: {
								draggable: false,
								selfIntersectable: false,
								coordinates: {
									draggable: true,
									midpoints: true,
									deletable: true,
									snappable: snapping
								}
							}
						}
					},
					styles: {
						selectedPolygonColor: FILL,
						selectedPolygonFillOpacity: 0.35,
						selectedPolygonOutlineColor: OUTLINE,
						selectedPolygonOutlineWidth: 3,
						selectionPointColor: HANDLE,
						selectionPointOutlineColor: FILL,
						selectionPointWidth: 6,
						selectionPointOutlineWidth: 2,
						midPointColor: FILL,
						midPointOutlineColor: HANDLE,
						midPointWidth: 4,
						midPointOutlineWidth: 1
					}
				})
			]
		});
		created.start();
		created.on('change', report);
		created.on('select', report);
		created.on('deselect', report);
		created.on('finish', (id, context) => {
			// A finished part can be adjusted right away.
			if (context.mode === 'polygon' && context.action === 'draw') {
				created.setMode('select');
				created.selectFeature(id);
			}
			report();
		});
		draw = created;
	}

	// Creates the map in the browser; mapbox-gl can't run on the server.
	$effect(() => {
		if (!token || !container) return;
		const element = container;
		let cancelled = false;
		let created: MapboxMap | undefined;
		import('mapbox-gl').then(({ default: mapboxgl }) => {
			if (cancelled) return;
			const initial = bounds(corners(areas)) ?? bounds(fallback);
			created = new mapboxgl.Map({
				container: element,
				accessToken: token,
				style: 'mapbox://styles/mapbox/satellite-streets-v12',
				attributionControl: false,
				// An explicit camera, or Mapbox jumps to the style's own view.
				center: [0, 0],
				zoom: 1,
				...(initial ? { bounds: initial, fitBoundsOptions: FIT } : {})
			});
			created.addControl(new mapboxgl.NavigationControl({ showCompass: false }), 'top-right');
			created.addControl(new mapboxgl.ScaleControl({ unit: 'metric' }), 'bottom-left');
			created.addControl(new mapboxgl.AttributionControl({ compact: true }), 'bottom-right');
			const instance = created;
			instance.on('style.load', () => addLayers(instance));
			instance.on('error', (event) => {
				const status = (event.error as { status?: number } | undefined)?.status;
				if (status === 401 || status === 403 || !instance.isStyleLoaded()) failed = true;
			});
			instance.on('click', 'area-fill', (event) => {
				// mapbox-gl's feature type leaves out the properties.
				const feature = event.features?.[0] as { properties?: { id?: unknown } } | undefined;
				const id = feature?.properties?.id;
				if (typeof id === 'string' && !editing) onselect?.(id);
			});
			instance.on('mouseenter', 'area-fill', () => {
				if (!editing) instance.getCanvas().style.cursor = 'pointer';
			});
			instance.on('mouseleave', 'area-fill', () => (instance.getCanvas().style.cursor = ''));
			instance.once('load', () => {
				if (!cancelled) startDrawing(instance);
			});
			map = instance;
		});
		return () => {
			cancelled = true;
			draw?.stop();
			draw = undefined;
			created?.remove();
			map = undefined;
		};
	});

	// The areas, the selection or the area being edited changed.
	$effect(() => {
		const data = shown(areas, editing, selected);
		const names = labels(areas, editing);
		(map?.getSource('areas') as GeoJSONSource | undefined)?.setData(data);
		(map?.getSource('area-labels') as GeoJSONSource | undefined)?.setData(names);
	});

	// A newly selected area comes into view.
	$effect(() => {
		const area = areas.find((area) => area.id === selected);
		if (!map || !area) return;
		const extent = bounds(parts(area.boundary).flatMap((rings) => rings[0]));
		if (extent) map.fitBounds(extent, { ...FIT, animate: !reducedMotion() });
	});

	// Editing starts or ends: Terra Draw gets the area's parts, or lets go.
	$effect(() => {
		const id = editing;
		if (!draw) return;
		draw.clear();
		if (id === null) {
			draw.setMode('static');
		} else if (id === 'new') {
			draw.setMode('polygon');
		} else {
			const area = areas.find((area) => area.id === id);
			// Only the outer rings: holes are where other areas are, and clipping
			// on save puts them back. Terra Draw takes at most 9 decimals.
			const round = (value: number) => Math.round(value * 1e9) / 1e9;
			const features = area
				? parts(area.boundary).map((rings) => ({
						type: 'Feature' as const,
						properties: { mode: 'polygon' },
						geometry: {
							type: 'Polygon' as const,
							coordinates: [rings[0].map(([lon, lat]) => [round(lon), round(lat)])]
						}
					}))
				: [];
			for (const result of draw.addFeatures(features)) {
				if (!result.valid) console.error(`Terra Draw refused a part: ${result.reason}`);
			}
			draw.setMode('select');
		}
		report();
	});

	/** Starts drawing another part. */
	export function drawPart() {
		draw?.setMode('polygon');
		report();
	}

	/** Removes the selected part. */
	export function removePart() {
		if (!draw) return;
		const ids = draw
			.getSnapshot()
			.filter((feature) => feature.properties.selected)
			.map((feature) => feature.id as string);
		draw.removeFeatures(ids);
		report();
	}

	/** The parts drawn so far, as one boundary; null without any. */
	export function boundary(): Boundary | null {
		if (!draw) return null;
		const polygons = draw
			.getSnapshot()
			.filter(
				(feature) =>
					feature.geometry.type === 'Polygon' &&
					feature.properties.mode === 'polygon' &&
					!feature.properties.currentlyDrawing
			)
			.map((feature) => (feature.geometry as { coordinates: number[][][] }).coordinates);
		if (polygons.length === 0) return null;
		return { type: 'MultiPolygon', coordinates: polygons };
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
	<div class="map" bind:this={container} role="region" aria-label={label}></div>
{/if}

<style>
	.map {
		height: max(30rem, calc(100vh - 12rem));
		background: var(--cds-layer-01);
	}
</style>
