//! Derives a flight from a log's frames: when it was in the air, how far it flew,
//! and its track.

use chrono::{DateTime, TimeDelta, Utc};
use dji_log_parser::frame::Frame;
use dji_log_parser::layout::details::Details;
use serde::Serialize;
use serde_json::{Value, json};

/// Time between track points.
const TRACK_INTERVAL: TimeDelta = TimeDelta::seconds(1);
const EARTH_RADIUS_M: f64 = 6_371_008.8;

pub struct Flight {
    pub takeoff_at: DateTime<Utc>,
    pub landed_at: DateTime<Utc>,
    pub air_time_s: i32,
    pub distance_m: f64,
    pub aircraft_serial: Option<String>,
    pub rc_serial: Option<String>,
    /// GeoJSON LineString: [longitude, latitude, height above takeoff].
    pub track: Value,
    pub metrics: Metrics,
}

/// Display-only summaries, stored as JSON.
#[derive(Serialize)]
pub struct Metrics {
    pub max_altitude_m: f64,
    pub camera_serial: Option<String>,
    pub battery_serial: Option<String>,
}

/// Serial numbers from the summary in a log.
#[derive(Default)]
pub struct Serials {
    pub aircraft: Option<String>,
    pub rc: Option<String>,
    pub camera: Option<String>,
    pub battery: Option<String>,
}

impl Serials {
    pub fn from_details(details: &Details) -> Self {
        Self {
            aircraft: non_empty(&details.aircraft_sn),
            rc: non_empty(&details.rc_sn),
            camera: non_empty(&details.camera_sn),
            battery: non_empty(&details.battery_sn),
        }
    }
}

/// The flight in a log, or `None` if the aircraft never left the ground.
pub fn derive(frames: &[Frame], serials: Serials) -> Option<Flight> {
    let airborne: Vec<&Frame> = frames.iter().filter(|f| !f.osd.is_on_ground).collect();
    let (first, last) = (airborne.first()?, airborne.last()?);

    // Time and distance between consecutive frames in the air. Gaps in the recording
    // count (the aircraft was flying on both sides); time on the ground doesn't.
    let mut air_time = TimeDelta::zero();
    for pair in frames.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if !a.osd.is_on_ground && !b.osd.is_on_ground {
            air_time += (b.custom.date_time - a.custom.date_time).max(TimeDelta::zero());
        }
    }
    let mut distance_m = 0.0;
    let mut previous: Option<&Frame> = None;
    for frame in frames {
        if frame.osd.is_on_ground {
            previous = None;
        } else if has_position(frame) {
            if let Some(previous) = previous {
                distance_m += haversine_m(previous, frame);
            }
            previous = Some(frame);
        }
    }

    let max_altitude_m = airborne
        .iter()
        .map(|f| f64::from(f.osd.height))
        .fold(0.0, f64::max);

    Some(Flight {
        takeoff_at: first.custom.date_time,
        landed_at: last.custom.date_time,
        air_time_s: air_time.num_seconds() as i32,
        distance_m,
        aircraft_serial: serials.aircraft,
        rc_serial: serials.rc,
        track: track(&airborne),
        metrics: Metrics {
            max_altitude_m: round(max_altitude_m, 1),
            camera_serial: serials.camera,
            battery_serial: serials.battery,
        },
    })
}

/// One point per `TRACK_INTERVAL`, plus the last airborne position.
fn track(airborne: &[&Frame]) -> Value {
    let positioned: Vec<&Frame> = airborne
        .iter()
        .copied()
        .filter(|f| has_position(f))
        .collect();
    let mut points = Vec::new();
    let mut next_at = None;
    for (i, frame) in positioned.iter().enumerate() {
        let is_last = i + 1 == positioned.len();
        if next_at.is_none_or(|at| frame.custom.date_time >= at) || is_last {
            points.push(json!([
                round(frame.osd.longitude, 7),
                round(frame.osd.latitude, 7),
                round(f64::from(frame.osd.height), 1),
            ]));
            next_at = Some(frame.custom.date_time + TRACK_INTERVAL);
        }
    }
    json!({ "type": "LineString", "coordinates": points })
}

/// Whether the frame has a GPS position; without a fix the parser reports 0, 0.
fn has_position(frame: &Frame) -> bool {
    let (lat, lon) = (frame.osd.latitude, frame.osd.longitude);
    (lat != 0.0 || lon != 0.0) && lat.abs() <= 90.0 && lon.abs() <= 180.0
}

fn haversine_m(a: &Frame, b: &Frame) -> f64 {
    let (lat1, lat2) = (a.osd.latitude.to_radians(), b.osd.latitude.to_radians());
    let d_lat = lat2 - lat1;
    let d_lon = (b.osd.longitude - a.osd.longitude).to_radians();
    let h = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().asin()
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn round(value: f64, decimals: i32) -> f64 {
    let factor = 10f64.powi(decimals);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(seconds: f64, on_ground: bool, lat: f64, lon: f64, height: f32) -> Frame {
        let mut frame = Frame::default();
        frame.custom.date_time =
            DateTime::from_timestamp_millis((seconds * 1000.0) as i64).unwrap();
        frame.osd.is_on_ground = on_ground;
        frame.osd.latitude = lat;
        frame.osd.longitude = lon;
        frame.osd.height = height;
        frame
    }

    /// A flight north along a meridian, frames at 10 Hz, 0.0001° of latitude (about
    /// 11.1 m) per second.
    fn flight(seconds: usize) -> Vec<Frame> {
        let mut frames = vec![frame(0.0, true, 0.0, 0.0, 0.0)];
        for i in 1..=seconds * 10 {
            let t = i as f64 / 10.0;
            frames.push(frame(t, false, t * 0.0001, 10.0, 50.0));
        }
        frames.push(frame(seconds as f64 + 0.1, true, 0.0, 0.0, 0.0));
        frames
    }

    #[test]
    fn no_flight_without_takeoff() {
        let frames = vec![
            frame(0.0, true, 0.0, 0.0, 0.0),
            frame(1.0, true, 0.0, 0.0, 0.0),
        ];
        assert!(derive(&frames, Serials::default()).is_none());
    }

    #[test]
    fn times_distance_and_track() {
        let flight = derive(&flight(60), Serials::default()).unwrap();
        assert_eq!(flight.takeoff_at.timestamp_millis(), 100);
        assert_eq!(flight.landed_at.timestamp_millis(), 60_000);
        assert_eq!(flight.air_time_s, 59);
        // 59.9 s at 0.0001°/s of latitude.
        let expected = 59.9 * 0.0001_f64.to_radians() * EARTH_RADIUS_M;
        assert!(
            (flight.distance_m - expected).abs() < 0.5,
            "{}",
            flight.distance_m
        );
        assert_eq!(flight.metrics.max_altitude_m, 50.0);

        let points = flight.track["coordinates"].as_array().unwrap();
        // One per second from 0.1 s to 59.1 s, plus the last position at 60 s.
        assert_eq!(points.len(), 61);
        assert_eq!(points[0], json!([10.0, 0.00001, 50.0]));
    }

    #[test]
    fn recording_gaps_in_the_air_are_air_time() {
        let mut frames = flight(10);
        // No frames for 30 seconds while flying, as happens in real logs.
        for frame in frames.iter_mut().skip(51) {
            frame.custom.date_time += TimeDelta::seconds(30);
        }
        let flight = derive(&frames, Serials::default()).unwrap();
        assert_eq!(flight.air_time_s, 39);
    }

    #[test]
    fn time_on_the_ground_between_takeoffs_is_not_air_time() {
        let mut frames = flight(10);
        // Landed for 2 minutes halfway, then took off again.
        for frame in frames.iter_mut().skip(51) {
            frame.custom.date_time += TimeDelta::minutes(2);
        }
        let landing_at = frames[50].custom.date_time + TimeDelta::seconds(1);
        frames.insert(51, frame(0.0, true, 0.0, 0.0, 0.0));
        frames[51].custom.date_time = landing_at;
        let flight = derive(&frames, Serials::default()).unwrap();
        assert_eq!(flight.air_time_s, 9);
        assert_eq!((flight.landed_at - flight.takeoff_at).num_seconds(), 129);
    }

    #[test]
    fn positions_without_gps_are_skipped() {
        let mut frames = flight(10);
        frames[20].osd.latitude = 0.0;
        frames[20].osd.longitude = 0.0;
        let with_gap = derive(&frames, Serials::default()).unwrap();
        let without_gap = derive(&flight(10), Serials::default()).unwrap();
        // No jump to 0, 0: the distance bridges the missing fix in a straight line.
        assert!((with_gap.distance_m - without_gap.distance_m).abs() < 0.01);
    }
}
