<!--
	@component
	A Mapbox map of the organisation's areas, for managing them. Every area is
	an outline with its name; clicking one selects it, clicking elsewhere
	deselects it. The drawing tools are a Mapbox control group under the zoom
	buttons, in Mapbox's style (user's decision): New area draws a polygon,
	named at the shape when it's closed; Edit shape makes the selected area's
	corners draggable (right-click removes one). Drawing is Terra Draw, which
	snaps to the corners and edges of other areas. Phones only view.
-->
<script lang="ts">
	import 'mapbox-gl/dist/mapbox-gl.css';
	import AreaIcon from '@carbon/icons/es/area/16.js';
	import EditIcon from '@carbon/icons/es/edit/16.js';
	import type { IControl, Map as MapboxMap } from 'mapbox-gl';
	import type { TerraDraw, TerraDrawMouseEvent } from 'terra-draw';
	import {
		FILL,
		OUTLINE,
		addAreaLayers,
		bounds,
		corners,
		setAreas,
		type Area,
		type Boundary
	} from './areas.js';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';
	import InlineNotification from './InlineNotification.svelte';
	import TextInput from './TextInput.svelte';

	type Polygon = Extract<Boundary, { type: 'Polygon' }>;

	interface Props {
		areas: Area[];
		/** The install's public Mapbox token; without one there is no map. */
		token: string;
		/** For screen readers, e.g. "Map of the areas". */
		label: string;
		selected?: string | null;
		/** Where to look when there are no areas yet, e.g. a recent flight's track. */
		fallback?: number[][];
		/** An area was clicked, or (null) the map outside the areas. */
		onselect?: (id: string | null) => void;
		/** Drawing or editing started or ended, e.g. to lock a list meanwhile. */
		onbusy?: (busy: boolean) => void;
		/** Saves a new area; resolves to an error message, if refused. */
		oncreate: (name: string, boundary: Polygon) => Promise<string | undefined>;
		/** Saves a changed shape; resolves to an error message, if refused. */
		onreshape: (id: string, boundary: Polygon) => Promise<string | undefined>;
	}

	let {
		areas,
		token,
		label,
		selected = null,
		fallback = [],
		onselect,
		onbusy,
		oncreate,
		onreshape
	}: Props = $props();

	// Through a derived, which only signals a real change: reading the prop
	// itself tracks the parent's whole `data`, so every reload of the page's
	// data (e.g. after saving an area) would create the map again.
	const mapToken = $derived(token);

	type Mode = 'view' | 'drawing' | 'naming' | 'editing';
	let mode: Mode = $state('view');
	let container: HTMLDivElement | undefined = $state();
	let tools: HTMLDivElement | undefined = $state();
	let map: MapboxMap | undefined = $state.raw();
	let draw: TerraDraw | undefined = $state.raw();
	let failed = $state(false);
	let saving = $state(false);
	let error: string | undefined = $state();
	let name = $state('');
	/** Where the name field sits: at the drawn shape, in map pixels. */
	let at: { x: number; y: number } | null = $state(null);

	// Terra Draw's handles: white with a Blue 60 outline.
	const HANDLE = '#ffffff';
	/** How close (in pixels) a corner must come to another area to snap to it. */
	const SNAP_PX = 6;
	/** How near (in pixels) the pointer must be to a corner to close or grab it. */
	const POINTER_PX = 8;
	const FIT = { padding: 48, maxZoom: 17 };

	$effect(() => onbusy?.(mode !== 'view'));

	// The name field takes focus when it appears (browsers may block autofocus).
	let naming: HTMLFormElement | undefined = $state();
	$effect(() => {
		naming?.querySelector('input')?.focus();
	});

	/** The areas shown as Mapbox layers: all but the one being edited. */
	function shown(areas: Area[], mode: Mode, selected: string | null) {
		return mode === 'editing' ? areas.filter((area) => area.id !== selected) : areas;
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
		const rings = shown(areas, mode, selected).map((area) => area.boundary.coordinates[0]);
		let best: number[] | undefined;
		let nearest = SNAP_PX;
		for (const ring of rings as number[][][]) {
			for (const corner of ring) {
				const p = context.project(corner[0], corner[1]);
				const distance = Math.hypot(p.x - pointer.x, p.y - pointer.y);
				if (distance < nearest) [best, nearest] = [corner, distance];
			}
		}
		if (best) return best;
		for (const ring of rings as number[][][]) {
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

	async function startDrawing(instance: MapboxMap) {
		const [{ TerraDraw, TerraDrawPolygonMode, TerraDrawSelectMode }, { TerraDrawMapboxGLAdapter }] =
			await Promise.all([import('terra-draw'), import('terra-draw-mapbox-gl-adapter')]);
		const snapping = { toCustom: snapToNeighbours };
		const created = new TerraDraw({
			adapter: new TerraDrawMapboxGLAdapter({ map: instance }),
			modes: [
				new TerraDrawPolygonMode({
					snapping,
					// How near a click must be to the first corner to close the shape
					// (Terra Draw's default of 40px pulled clicks onto the drawing).
					pointerDistance: POINTER_PX,
					cursors: { start: 'crosshair', close: 'pointer' },
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
					pointerDistance: POINTER_PX,
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
		created.on('finish', (_id, context) => {
			// A closed shape is named next, at the shape.
			if (context.mode === 'polygon' && context.action === 'draw') {
				created.setMode('static');
				mode = 'naming';
				name = '';
				error = undefined;
				place();
			}
		});
		draw = created;
	}

	/** The polygon drawn or edited, if any. */
	function drawn(): Polygon | null {
		const feature = draw
			?.getSnapshot()
			.find(
				(feature) => feature.geometry.type === 'Polygon' && !feature.properties.currentlyDrawing
			);
		return feature ? (feature.geometry as Polygon) : null;
	}

	/** Puts the name field above the drawn shape's top, kept on the map (it
	 * is about 11rem high). */
	function place() {
		const shape = drawn();
		const extent = shape && bounds(shape.coordinates[0]);
		if (!map || !extent || !container) return;
		const [[west], [east, north]] = extent;
		const point = map.project([(west + east) / 2, north]);
		at = {
			x: Math.min(Math.max(point.x, 140), container.clientWidth - 140),
			y: Math.min(Math.max(point.y, 184), container.clientHeight - 8)
		};
	}

	function stop() {
		draw?.clear();
		draw?.setMode('static');
		if (map) map.getCanvas().style.cursor = '';
		mode = 'view';
		error = undefined;
		at = null;
	}

	/** Starts drawing a new area, or stops if already drawing; also for a
	 * button outside the map. */
	export function newArea() {
		if (!draw) return;
		if (mode === 'drawing') return stop();
		stop();
		draw.setMode('polygon');
		mode = 'drawing';
	}

	function editShape() {
		const area = areas.find((area) => area.id === selected);
		if (!draw || !area || mode !== 'view') return;
		// Terra Draw takes at most 9 decimals.
		const round = (value: number) => Math.round(value * 1e9) / 1e9;
		const [result] = draw.addFeatures([
			{
				type: 'Feature',
				properties: { mode: 'polygon' },
				geometry: {
					type: 'Polygon',
					// Only the outer ring: holes are where other areas are, and
					// clipping on save puts them back.
					coordinates: [
						(area.boundary.coordinates[0] as number[][]).map(([lon, lat]) => [
							round(lon),
							round(lat)
						])
					]
				}
			}
		]);
		if (!result.valid) {
			console.error(`Terra Draw refused the shape: ${result.reason}`);
			return;
		}
		draw.setMode('select');
		draw.selectFeature(result.id as string);
		mode = 'editing';
		error = undefined;
	}

	async function save() {
		const shape = drawn();
		if (!shape || saving) return;
		saving = true;
		const refused =
			mode === 'naming'
				? await oncreate(name, shape)
				: selected
					? await onreshape(selected, shape)
					: undefined;
		saving = false;
		if (refused) error = refused;
		else stop();
	}

	function keydown(event: KeyboardEvent) {
		if (event.key !== 'Escape') return;
		if (mode === 'view' && selected) onselect?.(null);
		else if (mode !== 'view') stop();
	}

	// Creates the map in the browser; mapbox-gl can't run on the server.
	$effect(() => {
		const accessToken = mapToken;
		if (!accessToken || !container || !tools) return;
		const element = container;
		const toolGroup = tools;
		let cancelled = false;
		let created: MapboxMap | undefined;
		import('mapbox-gl').then(({ default: mapboxgl }) => {
			if (cancelled) return;
			const initial = bounds(corners(areas)) ?? bounds(fallback);
			created = new mapboxgl.Map({
				container: element,
				accessToken,
				style: 'mapbox://styles/mapbox/satellite-streets-v12',
				attributionControl: false,
				// An explicit camera, or Mapbox jumps to the style's own view.
				center: [0, 0],
				zoom: 1,
				...(initial ? { bounds: initial, fitBoundsOptions: FIT } : {})
			});
			created.addControl(new mapboxgl.NavigationControl({ showCompass: false }), 'top-right');
			// The drawing tools, under the zoom buttons.
			const control: IControl = { onAdd: () => toolGroup, onRemove: () => toolGroup.remove() };
			created.addControl(control, 'top-right');
			created.addControl(new mapboxgl.ScaleControl({ unit: 'metric' }), 'bottom-left');
			created.addControl(new mapboxgl.AttributionControl({ compact: true }), 'bottom-right');
			const instance = created;
			instance.on('style.load', () =>
				addAreaLayers(instance, shown(areas, mode, selected), selected)
			);
			instance.on('error', (event) => {
				const status = (event.error as { status?: number } | undefined)?.status;
				if (status === 401 || status === 403 || !instance.isStyleLoaded()) failed = true;
			});
			instance.on('click', (event) => {
				if (mode !== 'view' || !instance.getLayer('area-fill')) return;
				// mapbox-gl's feature type leaves out the properties.
				const [feature] = instance.queryRenderedFeatures(event.point, {
					layers: ['area-fill']
				}) as { properties?: { id?: unknown } }[];
				const id = feature?.properties?.id;
				onselect?.(typeof id === 'string' ? id : null);
			});
			// A pointer over an area that can be selected; while drawing or
			// editing, Terra Draw sets the cursor.
			instance.on('mousemove', (event) => {
				if (mode !== 'view' || !instance.getLayer('area-fill')) return;
				const over =
					instance.queryRenderedFeatures(event.point, { layers: ['area-fill'] }).length > 0;
				instance.getCanvas().style.cursor = over ? 'pointer' : '';
			});
			instance.on('move', () => {
				if (mode === 'naming') place();
			});
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
		const visible = shown(areas, mode, selected);
		if (map) setAreas(map, visible, selected);
	});

	// A newly selected area comes into view, if it isn't already.
	$effect(() => {
		const area = areas.find((area) => area.id === selected);
		if (!map || !area) return;
		const ring = area.boundary.coordinates[0] as number[][];
		const view = map.getBounds();
		if (view && ring.every(([lon, lat]) => view.contains([lon, lat]))) return;
		const extent = bounds(ring);
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (extent) map.fitBounds(extent, { ...FIT, animate: !still });
	});
</script>

<svelte:window onkeydown={keydown} />

{#if !token}
	<InlineNotification kind="info" title="No map">
		This install has no Mapbox token. An administrator can set MAPBOX_ACCESS_TOKEN.
	</InlineNotification>
{:else if failed}
	<InlineNotification kind="error" title="The map couldn’t load">
		Mapbox didn’t answer or refused the token. Try again later.
	</InlineNotification>
{/if}
<div class="frame" class:hidden={!token || failed}>
	<div class="map" bind:this={container} role="region" aria-label={label}></div>

	<!-- Mapbox's control group; the map moves it under its zoom buttons. -->
	<div class="mapboxgl-ctrl mapboxgl-ctrl-group area-tools" bind:this={tools}>
		<button
			type="button"
			class:active={mode === 'drawing'}
			aria-pressed={mode === 'drawing'}
			title="New area"
			aria-label="New area"
			disabled={mode === 'naming' || mode === 'editing'}
			onclick={newArea}><Icon icon={AreaIcon} /></button
		>
		<button
			type="button"
			class:active={mode === 'editing'}
			aria-pressed={mode === 'editing'}
			title="Edit shape"
			aria-label="Edit shape"
			disabled={!selected || mode !== 'view'}
			onclick={editShape}><Icon icon={EditIcon} /></button
		>
	</div>

	{#if mode === 'editing'}
		<div class="bar">
			{#if error}<InlineNotification kind="error" title={error} />{/if}
			<div class="buttons">
				<Button kind="secondary" size="sm" onclick={stop}>Cancel</Button>
				<Button size="sm" disabled={saving} onclick={save}>Save</Button>
			</div>
		</div>
	{/if}

	{#if mode === 'naming' && at}
		<form
			class="naming layer-01"
			bind:this={naming}
			style:left="{at.x}px"
			style:top="{at.y}px"
			aria-label="Name the new area"
			onsubmit={(event) => {
				event.preventDefault();
				save();
			}}
		>
			<TextInput label="Name" size="sm" bind:value={name} {error} required />
			<div class="buttons">
				<Button kind="secondary" size="sm" onclick={stop}>Cancel</Button>
				<Button type="submit" size="sm" disabled={saving}>Save</Button>
			</div>
		</form>
	{/if}
</div>

<style>
	.frame {
		position: relative;
	}

	.hidden {
		display: none;
	}

	.map {
		height: max(30rem, calc(100vh - 12rem));
		background: var(--cds-layer-01);
	}

	/* Mapbox's own buttons hold an image; these hold a Carbon icon, centred. */
	.area-tools button {
		display: flex;
		align-items: center;
		justify-content: center;
		color: #333;
	}

	/* The tool in use, as Mapbox marks a pressed control. */
	.area-tools button.active {
		background-color: rgb(0 0 0 / 0.1);
	}

	.area-tools button:disabled {
		color: #aaa;
	}

	/* Phones only view the areas. */
	@media (max-width: 41.98rem) {
		.area-tools {
			display: none;
		}
	}

	/* Save and Cancel while editing, at the top of the map. */
	.bar {
		position: absolute;
		top: var(--cds-spacing-05);
		left: 50%;
		display: grid;
		gap: var(--cds-spacing-03);
		width: max-content;
		max-width: calc(100% - 8rem);
		transform: translateX(-50%);
	}

	.buttons {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
	}

	/* The new area's name, on a layer at the shape. */
	.naming {
		position: absolute;
		display: grid;
		gap: var(--cds-spacing-05);
		width: 16rem;
		padding: var(--cds-spacing-05);
		transform: translate(-50%, calc(-100% - var(--cds-spacing-03)));
		background: var(--cds-layer-01);
		box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.3);
	}
</style>
