<!--
	@component
	A Mapbox map of the organisation's areas, for managing them: every area as
	an outline with its name, and the one being drawn or edited in Terra Draw.
	Drawing snaps to the corners and edges of the other areas, so neighbours
	can share a border. Mapbox and Terra Draw aren't part of Carbon; the
	colours are Carbon's.
-->
<script lang="ts" module>
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
	import type { Map as MapboxMap } from 'mapbox-gl';
	import type { TerraDraw, TerraDrawMouseEvent } from 'terra-draw';
	import {
		FILL,
		OUTLINE,
		addAreaLayers,
		bounds,
		corners,
		parts,
		setAreas,
		type Area,
		type Boundary
	} from './areas.js';
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

	// Terra Draw's handles: white with a Blue 60 outline.
	const HANDLE = '#ffffff';
	/** How close (in pixels) a corner must come to another area to snap to it. */
	const SNAP_PX = 6;

	/** The areas shown as Mapbox layers: all but the one being edited. */
	function shown(areas: Area[], editing: string | null) {
		return areas.filter((area) => area.id !== editing);
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
		// Only to other areas: Terra Draw's own snapping to the area's corners
		// reaches 40px, which pulled every nearby corner onto them.
		const snapping = { toCustom: snapToNeighbours };
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
			instance.on('style.load', () => addAreaLayers(instance, shown(areas, editing), selected));
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
		const visible = shown(areas, editing);
		if (map) setAreas(map, visible, selected);
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
