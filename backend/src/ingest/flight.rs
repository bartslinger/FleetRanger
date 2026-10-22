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
    /// None if the log names no battery.
    pub battery: Option<BatteryUse>,
}

/// What the battery's static record in a log says.
#[derive(Default, Clone, Copy)]
pub struct BatteryInfo {
    pub design_capacity_mah: Option<i32>,
    pub cycles: Option<i32>,
}

/// A battery on one flight; what the log doesn't report is None.
pub struct BatteryUse {
    pub serial: String,
    pub charge_start_pct: Option<i16>,
    pub charge_end_pct: Option<i16>,
    pub full_capacity_mah: Option<i32>,
    pub design_capacity_mah: Option<i32>,
    pub cycles: Option<i32>,
    pub temperature_min_c: Option<f32>,
    pub temperature_max_c: Option<f32>,
    pub cell_voltage_min_v: Option<f32>,
    pub cell_spread_max_v: Option<f32>,
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
pub fn derive(frames: &[Frame], serials: Serials, battery_info: BatteryInfo) -> Option<Flight> {
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
        battery: serials
            .battery
            .clone()
            .map(|serial| battery(&airborne, serial, battery_info)),
        metrics: Metrics {
            max_altitude_m: round(max_altitude_m, 1),
            camera_serial: serials.camera,
            battery_serial: serials.battery,
        },
    })
}

/// The battery's state from takeoff to landing. Frames before the battery's
/// first report have 0 V; capacity and temperature come in one record, which
/// older logs don't have; cells are left out while the parser estimates them
/// from the total voltage, or while one hasn't reported yet (0 V).
fn battery(airborne: &[&Frame], serial: String, info: BatteryInfo) -> BatteryUse {
    let reported: Vec<_> = airborne
        .iter()
        .map(|f| &f.battery)
        .filter(|b| b.voltage > 0.0)
        .collect();
    let with_capacity: Vec<_> = reported.iter().filter(|b| b.full_capacity > 0).collect();
    let cells: Vec<(f32, f32)> = reported
        .iter()
        .filter(|b| !b.is_cell_voltage_estimated && b.cell_voltages.iter().all(|&v| v > 0.0))
        .filter_map(|b| {
            let min = b.cell_voltages.iter().copied().reduce(f32::min)?;
            let max = b.cell_voltages.iter().copied().reduce(f32::max)?;
            Some((min, max - min))
        })
        .collect();
    let temperatures = with_capacity.iter().map(|b| b.temperature);
    BatteryUse {
        serial,
        charge_start_pct: reported.first().map(|b| i16::from(b.charge_level)),
        charge_end_pct: reported.last().map(|b| i16::from(b.charge_level)),
        full_capacity_mah: with_capacity
            .last()
            .and_then(|b| i32::try_from(b.full_capacity).ok()),
        design_capacity_mah: info.design_capacity_mah,
        cycles: info.cycles,
        temperature_min_c: temperatures.clone().reduce(f32::min),
        temperature_max_c: temperatures.reduce(f32::max),
        cell_voltage_min_v: cells.iter().map(|c| c.0).reduce(f32::min),
        cell_spread_max_v: cells.iter().map(|c| c.1).reduce(f32::max),
    }
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
        assert!(derive(&frames, Serials::default(), BatteryInfo::default()).is_none());
    }

    #[test]
    fn times_distance_and_track() {
        let flight = derive(&flight(60), Serials::default(), BatteryInfo::default()).unwrap();
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
        let flight = derive(&frames, Serials::default(), BatteryInfo::default()).unwrap();
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
        let flight = derive(&frames, Serials::default(), BatteryInfo::default()).unwrap();
        assert_eq!(flight.air_time_s, 9);
        assert_eq!((flight.landed_at - flight.takeoff_at).num_seconds(), 129);
    }

    /// Charge from the battery's first to its last report in the air; cells
    /// only while all of them have reported and none is an estimate.
    #[test]
    fn battery_from_its_reports() {
        let mut frames = flight(10);
        for (i, frame) in frames.iter_mut().enumerate().skip(1).take(100) {
            let battery = &mut frame.battery;
            battery.voltage = 16.0;
            battery.charge_level = 100 - (i / 10) as u8;
            battery.full_capacity = 4_500;
            battery.temperature = 30.0 + (i / 10) as f32;
            battery.cell_voltages = vec![4.0, 3.9, 4.0, 4.0];
        }
        // No report yet: 0 V, and nothing else.
        frames[1].battery = Default::default();
        // One cell not reported yet, and cells estimated from the total.
        frames[2].battery.cell_voltages[3] = 0.0;
        frames[3].battery.cell_voltages = vec![3.0; 4];
        frames[3].battery.is_cell_voltage_estimated = true;
        let serials = Serials {
            battery: Some("TESTBATTERY1".to_owned()),
            ..Serials::default()
        };
        let info = BatteryInfo {
            design_capacity_mah: Some(5_000),
            cycles: Some(22),
        };

        let battery = derive(&frames, serials, info).unwrap().battery.unwrap();
        assert_eq!(battery.serial, "TESTBATTERY1");
        assert_eq!(
            (battery.charge_start_pct, battery.charge_end_pct),
            (Some(100), Some(90))
        );
        assert_eq!(battery.full_capacity_mah, Some(4_500));
        assert_eq!(
            (battery.design_capacity_mah, battery.cycles),
            (Some(5_000), Some(22))
        );
        assert_eq!(
            (battery.temperature_min_c, battery.temperature_max_c),
            (Some(30.0), Some(40.0))
        );
        assert_eq!(battery.cell_voltage_min_v, Some(3.9));
        assert!((battery.cell_spread_max_v.unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn positions_without_gps_are_skipped() {
        let mut frames = flight(10);
        frames[20].osd.latitude = 0.0;
        frames[20].osd.longitude = 0.0;
        let with_gap = derive(&frames, Serials::default(), BatteryInfo::default()).unwrap();
        let without_gap = derive(&flight(10), Serials::default(), BatteryInfo::default()).unwrap();
        // No jump to 0, 0: the distance bridges the missing fix in a straight line.
        assert!((with_gap.distance_m - without_gap.distance_m).abs() < 0.01);
    }
}
