//! The geometry of areas: reading and writing their GeoJSON boundaries,
//! clipping an area against the others, and dividing a flight's time and
//! distance over the areas its track passes through.

use geo::{
    BooleanOps, BoundingRect, ChamberlainDuquetteArea, Coord, Distance, Haversine, Intersects,
    LineString, MultiPolygon, Point, Polygon, Rect,
};
use serde::Deserialize;
use serde_json::{Value, json};

/// Parts left over from clipping that are smaller than this (in m²) are
/// slivers from borders that almost, but not exactly, coincide.
const SLIVER_M2: f64 = 1.0;

/// An area's boundary from GeoJSON: a Polygon or MultiPolygon in
/// [longitude, latitude]. None if it isn't one, or has no usable ring.
pub fn parse(value: &Value) -> Option<MultiPolygon> {
    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum Geometry {
        Polygon {
            coordinates: Vec<Vec<Vec<f64>>>,
        },
        MultiPolygon {
            coordinates: Vec<Vec<Vec<Vec<f64>>>>,
        },
    }
    let parts = match Geometry::deserialize(value).ok()? {
        Geometry::Polygon { coordinates } => vec![coordinates],
        Geometry::MultiPolygon { coordinates } => coordinates,
    };
    let mut polygons = Vec::new();
    for rings in parts {
        let mut rings = rings.into_iter().map(ring);
        let exterior = rings.next()??;
        let interiors = rings.collect::<Option<Vec<_>>>()?;
        polygons.push(Polygon::new(exterior, interiors));
    }
    (!polygons.is_empty()).then(|| MultiPolygon::new(polygons))
}

/// A closed ring of at least three distinct corners, all on the map.
fn ring(positions: Vec<Vec<f64>>) -> Option<LineString> {
    let mut coords = Vec::with_capacity(positions.len());
    for position in positions {
        let (&x, &y) = (position.first()?, position.get(1)?);
        if !(-180.0..=180.0).contains(&x) || !(-90.0..=90.0).contains(&y) {
            return None;
        }
        coords.push(Coord { x, y });
    }
    if coords.first() == coords.last() {
        coords.pop();
    }
    if coords.len() < 3 {
        return None;
    }
    let mut ring = LineString::new(coords);
    ring.close();
    Some(ring)
}

/// GeoJSON for a boundary: a Polygon if it has one part, else a MultiPolygon.
pub fn to_geojson(boundary: &MultiPolygon) -> Value {
    let polygon = |polygon: &Polygon| {
        std::iter::once(polygon.exterior())
            .chain(polygon.interiors())
            .map(|ring| ring.coords().map(|c| [c.x, c.y]).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    };
    match &boundary.0[..] {
        [one] => json!({ "type": "Polygon", "coordinates": polygon(one) }),
        parts => json!({
            "type": "MultiPolygon",
            "coordinates": parts.iter().map(polygon).collect::<Vec<_>>(),
        }),
    }
}

/// What is left of an area outside the others; None if nothing is. Only
/// the area itself changes, never the others (user's decision, 2026-10-21).
pub fn clip(area: &MultiPolygon, others: &[MultiPolygon]) -> Option<MultiPolygon> {
    // Without others, clipping still tidies a ring that crosses itself.
    let others = MultiPolygon::new(others.iter().flat_map(|other| other.0.clone()).collect());
    let left = area.difference(&others);
    let parts: Vec<Polygon> = left
        .0
        .into_iter()
        .filter(|part| part.chamberlain_duquette_unsigned_area() >= SLIVER_M2)
        .collect();
    (!parts.is_empty()).then(|| MultiPolygon::new(parts))
}

/// The indexes of the others that an area overlaps, so it would be clipped by them.
pub fn overlapping(area: &MultiPolygon, others: &[MultiPolygon]) -> Vec<usize> {
    (0..others.len())
        .filter(|&i| {
            area.intersects(&others[i])
                && area
                    .intersection(&others[i])
                    .chamberlain_duquette_unsigned_area()
                    >= SLIVER_M2
        })
        .collect()
}

/// A flight's time and distance in one area.
#[derive(Debug, PartialEq)]
pub struct Share {
    /// The area's index in the list the shares were computed for.
    pub area: usize,
    pub air_time_s: i32,
    pub distance_m: f64,
}

/// An area prepared for finding the points in it.
pub struct Shape {
    boundary: MultiPolygon,
    bounds: Option<Rect>,
}

impl Shape {
    pub fn new(boundary: MultiPolygon) -> Self {
        let bounds = boundary.bounding_rect();
        Self { boundary, bounds }
    }

    fn contains(&self, point: Point) -> bool {
        self.bounds.is_some_and(|bounds| bounds.intersects(&point))
            && self.boundary.intersects(&point)
    }
}

/// Divides a flight's air time and distance over the areas its track passes
/// through: each area gets the share of the track's points that lie in it
/// (the takeoff point is the first). The track has a point per second; its
/// distance between two points is split half and half over the areas of both.
/// Times are whole seconds that add up to the air time, with the part outside
/// all areas. An area the track touches gets a share even if it rounds to 0.
pub fn shares(track: &[Point], areas: &[Shape], air_time_s: i32, distance_m: f64) -> Vec<Share> {
    if track.is_empty() || areas.is_empty() {
        return Vec::new();
    }
    // Areas don't overlap, so a point is in one area at most; a point exactly
    // on a shared border goes to the first.
    let area_of: Vec<Option<usize>> = track
        .iter()
        .map(|&point| areas.iter().position(|area| area.contains(point)))
        .collect();

    // The last bucket is outside all areas.
    let outside = areas.len();
    let mut points = vec![0usize; areas.len() + 1];
    let mut lengths = vec![0.0; areas.len() + 1];
    for (i, area) in area_of.iter().enumerate() {
        points[area.unwrap_or(outside)] += 1;
        if let Some(next) = track.get(i + 1) {
            let half = Haversine.distance(track[i], *next) / 2.0;
            lengths[area.unwrap_or(outside)] += half;
            lengths[area_of[i + 1].unwrap_or(outside)] += half;
        }
    }

    let times = apportion(air_time_s.max(0), &points);
    let length: f64 = lengths.iter().sum();
    (0..areas.len())
        .filter(|&area| points[area] > 0)
        .map(|area| Share {
            area,
            air_time_s: times[area],
            distance_m: if length > 0.0 {
                distance_m * lengths[area] / length
            } else {
                0.0
            },
        })
        .collect()
}

/// Divides whole seconds in proportion to the counts, so that they add up:
/// each gets its share rounded down, and the seconds left over go to the
/// largest remainders (largest remainder method).
fn apportion(total: i32, counts: &[usize]) -> Vec<i32> {
    let sum: usize = counts.iter().sum();
    let exact: Vec<f64> = counts
        .iter()
        .map(|&count| f64::from(total) * count as f64 / sum as f64)
        .collect();
    let mut whole: Vec<i32> = exact.iter().map(|share| share.floor() as i32).collect();
    let mut by_remainder: Vec<usize> = (0..counts.len()).collect();
    by_remainder.sort_by(|&a, &b| {
        let remainder = |i: usize| exact[i] - f64::from(whole[i]);
        remainder(b).total_cmp(&remainder(a))
    });
    let left = total - whole.iter().sum::<i32>();
    for &i in by_remainder.iter().take(left.max(0) as usize) {
        whole[i] += 1;
    }
    whole
}

/// The positions of a stored track (a GeoJSON LineString), without heights.
pub fn track_points(track: &Value) -> Vec<Point> {
    track["coordinates"]
        .as_array()
        .map(|positions| {
            positions
                .iter()
                .filter_map(|position| {
                    Some(Point::new(
                        position.get(0)?.as_f64()?,
                        position.get(1)?.as_f64()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square of `size` degrees with its south-west corner at (x, y).
    fn square(x: f64, y: f64, size: f64) -> MultiPolygon {
        rectangle(x, y, size, size)
    }

    fn rectangle(x: f64, y: f64, width: f64, height: f64) -> MultiPolygon {
        let (east, north) = (x + width, y + height);
        parse(&json!({
            "type": "Polygon",
            "coordinates": [[[x, y], [east, y], [east, north], [x, north], [x, y]]],
        }))
        .unwrap()
    }

    fn area_m2(boundary: &MultiPolygon) -> f64 {
        boundary.chamberlain_duquette_unsigned_area()
    }

    #[test]
    fn clipping_cuts_the_new_area_back_to_its_neighbour() {
        let west = square(0.0, 0.0, 0.01);
        // Drawn generously over the border with `west`.
        let east = square(0.008, 0.0, 0.01);
        let clipped = clip(&east, std::slice::from_ref(&west)).unwrap();
        assert!((area_m2(&clipped) - area_m2(&rectangle(0.01, 0.0, 0.008, 0.01))).abs() < 1.0);
        assert!(
            clipped
                .intersection(&west)
                .0
                .iter()
                .all(|p| p.chamberlain_duquette_unsigned_area() < SLIVER_M2)
        );
    }

    #[test]
    fn clipping_leaves_a_hole_or_nothing() {
        let inner = square(0.004, 0.004, 0.002);
        let around = clip(&square(0.0, 0.0, 0.01), std::slice::from_ref(&inner)).unwrap();
        assert_eq!(around.0.len(), 1);
        assert_eq!(around.0[0].interiors().len(), 1);
        // An area entirely inside another one has nothing left.
        assert_eq!(clip(&inner, &[square(0.0, 0.0, 0.01)]), None);
    }

    #[test]
    fn shares_add_up_to_the_flight() {
        let areas = [
            Shape::new(square(0.0, 0.0, 0.01)),
            Shape::new(square(0.01, 0.0, 0.01)),
        ];
        // 7 points in the west area, 2 in the east one, 1 outside both.
        let mut track: Vec<Point> = (0..7)
            .map(|i| Point::new(0.001 * f64::from(i), 0.005))
            .collect();
        track.extend([
            Point::new(0.012, 0.005),
            Point::new(0.015, 0.005),
            Point::new(0.025, 0.005),
        ]);
        let shares = shares(&track, &areas, 100, 2_000.0);

        assert_eq!(shares.iter().map(|s| s.area).collect::<Vec<_>>(), [0, 1]);
        assert_eq!(shares[0].air_time_s, 70);
        assert_eq!(shares[1].air_time_s, 20);
        let distance: f64 = shares.iter().map(|s| s.distance_m).sum();
        assert!(
            distance < 2_000.0,
            "part of the distance is outside both areas"
        );
    }

    #[test]
    fn rounded_times_add_up_and_a_touched_area_keeps_its_share() {
        // A third each can't be whole seconds; they must still add up.
        assert_eq!(apportion(100, &[1, 1, 1]).iter().sum::<i32>(), 100);
        // A corner clipped for one point of 1000 gets a row, at 0 s.
        let areas = [
            Shape::new(square(0.0, 0.0, 0.01)),
            Shape::new(square(0.01, 0.0, 0.01)),
        ];
        let mut track = vec![Point::new(0.015, 0.005)];
        track.extend((0..999).map(|_| Point::new(0.005, 0.005)));
        let shares = shares(&track, &areas, 400, 0.0);
        assert_eq!(shares.len(), 2);
        assert_eq!(shares[1].air_time_s, 0);
        assert_eq!(shares[0].air_time_s, 400);
    }
}
