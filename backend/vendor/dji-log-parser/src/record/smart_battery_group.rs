use binrw::{binread, parser, BinResult};
use serde::Serialize;
use std::io::SeekFrom;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum SmartBatteryGroup {
    #[br(magic = 1u8)]
    SmartBatteryStatic(#[br(args { version })] SmartBatteryStatic),
    #[br(magic = 2u8)]
    SmartBatteryDynamic(#[br(args { version })] SmartBatteryDynamic),
    #[br(magic = 3u8)]
    SmartBatterySingleVoltage(#[br(args { version })] SmartBatterySingleVoltage),
}

/// Whether the record has an extra 0 byte after its index, which some records in version 14
/// logs have and others not: only if the next `length` bytes don't add up as the fields they
/// should be, and do with that byte skipped. Reads ahead and returns to where it started.
#[parser(reader)]
fn extra_byte(version: u8, length: usize, consistent: fn(&[u8]) -> bool) -> BinResult<bool> {
    if version < 14 {
        return Ok(false);
    }
    let start = reader.stream_position()?;
    let mut bytes = vec![0; length + 1];
    reader.read_exact(&mut bytes)?;
    reader.seek(SeekFrom::Start(start))?;
    Ok(bytes[0] == 0 && !consistent(&bytes[..length]) && consistent(&bytes[1..]))
}

/// A dynamic record's charge adds up: what remains is at most the full capacity, and the
/// charge percentage is what remains of it (within 2 points).
fn charge_adds_up(bytes: &[u8]) -> bool {
    let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let (full, remaining, percent) = (u32_at(8), u32_at(12), bytes[19]);
    full > 0
        && remaining <= full
        && (f64::from(percent) - f64::from(remaining) * 100.0 / f64::from(full)).abs() <= 2.0
}

/// A battery has cells.
fn has_cells(bytes: &[u8]) -> bool {
    bytes[0] > 0
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatteryStatic {
    pub index: u8,
    /// Version 14 logs have a byte here that version 11 logs don't.
    #[br(temp, if(version >= 14))]
    _unknown0: u8,
    pub designed_capacity: u32,
    pub loop_times: u16,
    pub full_voltage: u32,
    #[br(temp)]
    _unknown: u16,
    serial_number: u16,
    #[br(temp)]
    _unknown2: [u8; 10],
    #[br(temp)]
    _unknown3: [u8; 5],
    pub version_number: [u8; 8],
    pub battery_life: u8,
    pub battery_type: u8,
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatteryDynamic {
    pub index: u8,
    #[br(temp, parse_with = extra_byte, args(version, 20, charge_adds_up))]
    extra: bool,
    #[br(temp, if(extra))]
    _extra_byte: u8,
    /// volts
    #[br(map = |x: i32| x as f32 / 1000.0)]
    pub current_voltage: f32,
    // ampere
    #[br(map = |x: i32| x.abs() as f32 / 1000.0)]
    pub current_current: f32,
    /// mAh
    pub full_capacity: u32,
    /// mAh
    pub remained_capacity: u32,
    /// degrees
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub temperature: f32,
    pub cell_count: u8,
    pub capacity_percent: u8,
    pub battery_state: u64,
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatterySingleVoltage {
    pub index: u8,
    #[br(temp, parse_with = extra_byte, args(version, 1, has_cells))]
    extra: bool,
    #[br(temp, if(extra))]
    _extra_byte: u8,
    pub cell_count: u8,
    #[br(count = cell_count, map = |xs: Vec<u16>| xs.into_iter().map(|x| x as f32 / 1000.0).collect())]
    pub cell_voltages: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use std::io::Cursor;

    fn read(bytes: &[u8]) -> SmartBatteryGroup {
        let args = SmartBatteryGroupBinReadArgs { version: 14 };
        SmartBatteryGroup::read_args(&mut Cursor::new(bytes), args).unwrap()
    }

    /// A charge record as version 14 logs write it: 16.056 V, 6.878 A, 3,908 of 3,908 mAh
    /// full, 3,510 remaining, 24.6 degrees, 4 cells, 90%, with `extra` after the index.
    fn charge(extra: bool) -> Vec<u8> {
        let mut bytes = vec![2, 0];
        if extra {
            bytes.push(0);
        }
        bytes.extend(16056i32.to_le_bytes());
        bytes.extend((-6878i32).to_le_bytes());
        bytes.extend(3908u32.to_le_bytes());
        bytes.extend(3510u32.to_le_bytes());
        bytes.extend(246i16.to_le_bytes());
        bytes.extend([4, 90]);
        bytes.extend([0; 16]);
        bytes
    }

    #[test]
    fn charge_records_read_with_and_without_the_extra_byte() {
        for extra in [false, true] {
            let SmartBatteryGroup::SmartBatteryDynamic(battery) = read(&charge(extra)) else {
                panic!("not a charge record");
            };
            assert_eq!(
                (battery.current_voltage, battery.full_capacity, battery.temperature),
                (16.056, 3908, 24.6),
                "extra byte: {extra}"
            );
        }
    }

    /// A voltage whose low byte is 0 puts a 0 after the index too; the record still adds up
    /// as written, so nothing is skipped.
    #[test]
    fn charge_records_with_a_zero_after_the_index_that_belongs_to_them() {
        let mut bytes = charge(false);
        bytes[2..6].copy_from_slice(&15872i32.to_le_bytes());
        let SmartBatteryGroup::SmartBatteryDynamic(battery) = read(&bytes) else {
            panic!("not a charge record");
        };
        assert_eq!(battery.current_voltage, 15.872);
    }

    #[test]
    fn cell_records_read_with_and_without_the_extra_byte() {
        for bytes in [
            &[3, 0, 2, 0xe1, 0x0f, 0xda, 0x0f][..],
            &[3, 0, 0, 2, 0xe1, 0x0f, 0xda, 0x0f][..],
        ] {
            let SmartBatteryGroup::SmartBatterySingleVoltage(battery) = read(bytes) else {
                panic!("not a cell record");
            };
            assert_eq!(battery.cell_voltages, [4.065, 4.058]);
        }
    }
}
