// Areas on Mapbox maps: the areas page draws them for editing, flight and
// session maps for reference. Mapbox isn't part of Carbon; the colours are
// Carbon's.
import type { GeoJSONSource, Map as MapboxMap } from 'mapbox-gl';

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

// Map paint can't read CSS variables, so these are Carbon's values: Blue 50
// outlines with a white casing and a light Blue 60 fill, and names in white
// with a Gray 100 halo, readable on satellite imagery.
export const OUTLINE = '#4589ff';
export const FILL = '#0f62fe';
const CASING = '#ffffff';
const HALO = '#161616';

// mapbox-gl doesn't export its GeoJSON types.
type FeatureCollection = Extract<
	Parameters<GeoJSONSource['setData']>[0],
	{ type: 'FeatureCollection' }
>;

/** A boundary's parts, each as its rings. */
export function parts(boundary: Boundary): number[][][][] {
	return boundary.type === 'Polygon' ? [boundary.coordinates] : boundary.coordinates;
}

/** The extent of these positions as [[west, south], [east, north]]. */
export function bounds(positions: number[][]): [[number, number], [number, number]] | null {
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

/** The outer corners of these areas, e.g. to fit a map to them. */
export function corners(areas: Area[]): number[][] {
	return areas.flatMap((area) => parts(area.boundary).flatMap((rings) => rings[0]));
}

/** A point inside the largest part: the middle of the widest stretch of a
 * line across it, halfway between its north and south. */
function labelPoint(boundary: Boundary): number[] {
	const size = (ring: number[][]) => {
		const [[west, south], [east, north]] = bounds(ring) ?? [
			[0, 0],
			[0, 0]
		];
		return (east - west) * (north - south);
	};
	const rings = parts(boundary).reduce((a, b) => (size(b[0]) > size(a[0]) ? b : a));
	const [[west, south], [east, north]] = bounds(rings[0]) ?? [
		[0, 0],
		[0, 0]
	];
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

/** The areas as polygons, the selected one marked. */
function shapes(areas: Area[], selected: string | null): FeatureCollection {
	return {
		type: 'FeatureCollection',
		features: areas.map((area) => ({
			type: 'Feature' as const,
			properties: { id: area.id, name: area.name, selected: area.id === selected },
			geometry: area.boundary
		}))
	} as FeatureCollection;
}

/** One point per area for its name: Mapbox would label a polygon once per tile. */
function names(areas: Area[]): FeatureCollection {
	return {
		type: 'FeatureCollection',
		features: areas.map((area) => ({
			type: 'Feature' as const,
			properties: { name: area.name },
			geometry: { type: 'Point' as const, coordinates: labelPoint(area.boundary) }
		}))
	};
}

/** Adds the areas to the map's style: fills, outlines and names. The fill
 * layer is `area-fill`, e.g. for clicks. A `quiet` colour keeps them in the
 * background, as on a flight's map where the tracks matter: a thin neutral
 * border in that colour, without a fill. */
export function addAreaLayers(
	map: MapboxMap,
	areas: Area[],
	selected: string | null = null,
	quiet: string | null = null
) {
	map.addSource('areas', { type: 'geojson', data: shapes(areas, selected) });
	map.addSource('area-names', { type: 'geojson', data: names(areas) });
	map.addLayer({
		id: 'area-fill',
		type: 'fill',
		source: 'areas',
		paint: {
			'fill-color': FILL,
			'fill-opacity': quiet ? 0 : ['case', ['get', 'selected'], 0.35, 0.15]
		}
	});
	if (!quiet) {
		map.addLayer({
			id: 'area-casing',
			type: 'line',
			source: 'areas',
			// Rounded corners: Mapbox's default joins end sharp angles in long spikes.
			layout: { 'line-join': 'round' },
			paint: {
				'line-color': CASING,
				'line-width': ['case', ['get', 'selected'], 6, 4],
				'line-opacity': 0.8
			}
		});
	}
	map.addLayer({
		id: 'area-outline',
		type: 'line',
		source: 'areas',
		layout: { 'line-join': 'round' },
		paint: quiet
			? // Solid: shared borders would overlap, and dashes out of step look solid.
				{ 'line-color': quiet, 'line-width': 1 }
			: { 'line-color': OUTLINE, 'line-width': ['case', ['get', 'selected'], 4, 2] }
	});
	map.addLayer({
		id: 'area-name',
		type: 'symbol',
		source: 'area-names',
		layout: {
			'text-field': ['get', 'name'],
			'text-font': ['DIN Pro Medium', 'Arial Unicode MS Regular'],
			'text-size': quiet ? 12 : 14
		},
		paint: {
			'text-color': '#ffffff',
			'text-halo-color': HALO,
			'text-halo-width': quiet ? 1 : 1.5,
			'text-opacity': quiet ? 0.8 : 1
		}
	});
}

/** Shows other areas, or another selection, on a map that has the layers. */
export function setAreas(map: MapboxMap, areas: Area[], selected: string | null = null) {
	(map.getSource('areas') as GeoJSONSource | undefined)?.setData(shapes(areas, selected));
	(map.getSource('area-names') as GeoJSONSource | undefined)?.setData(names(areas));
}
