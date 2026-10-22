# Vendored dependencies

## dji-log-parser

The published [dji-log-parser 0.5.7](https://crates.io/crates/dji-log-parser/0.5.7), the latest
release, as unpacked from the `.crate` file (SHA-256
`13d15112d03932c3a4122c43db9604ec4378a41621193b29fb8f8b2e8502ba0b`). The upstream commit is in
`.cargo_vcs_info.json`. The crate ships without its license file, so `LICENSE` was copied from
that commit.

It is vendored so it can be patched. Each patch has its own section below, with the reason for
it. Drop a patch once upstream fixes the problem.

### Compiler warnings

Unnecessary parentheses in `#[br(map = …)]` closures are removed, and the auxiliary info's
version and signature, which are read but never used, are marked `temp`. This changes no
behaviour. Cargo shows warnings for vendored code, and these drowned out our own.

### Record types 253 and 254

In `src/keychain/feature_point.rs`, record types 253 and 254 are mapped to the DJIFlyCustom
feature point.

**Why.** Log records from version 13 on are AES-CBC encrypted per feature point, and each
record's IV follows from the previous record of the same feature point. Upstream doesn't know
types 253 and 254 and treats them as plaintext, so the DJIFlyCustom chain misses those records
and the next record in it gets the wrong IV. That record's first 16 bytes decrypt to garbage.
For a Custom record (type 5) that's the speed, distance and most of the timestamp, and the
frames take their date/time from the Custom records.

Type 253 comes once near the start of a log. Older logs have type-254 records now and then, so a
few Custom records broke without anyone noticing. The newer DJI app (from September 2026 on the
Mavic 3E) writes one before nearly every Custom record, so nearly all broke: frame times jumped
between 2015 and 2099, airborne time came out as 0, and end times landed decades ahead.

**Evidence.** A log from 6 Sept 2026 decrypts to 61 Custom records, 12 of them with a time
inside the flight; with the patch, 5,125, all inside the flight. A log from 2025 goes from
10,524 to 12,585 (all inside the flight). No other record type changes in number, and the
type-254 records themselves then decrypt to clean, structured data (`05 00 02 00 00 …`). With
253 as well, not a single record of a type the parser knows fails any more.

### Errors instead of silent garbage

In `src/lib.rs`, where upstream carries on without saying so:

- `DJILog::records` and `DJILog::keychains_request` stop at the first record they can't read
  and return what they have, so the rest of the log (or the keys it needs) goes missing. Here
  that's an error.
- `DJILog::keychains_request` reads every record to find the key ones, before it has any keys.
  An encrypted record between the key records (the aircraft's serial number, in 6 of 2,068
  logs tested) then reads as garbage, can run past its own end, and upstream loses its place
  and skips ahead past the remaining key records: half the keys never get requested, and the
  records they unlock decrypt to garbage. Here records other than the key ones are skipped by
  their length without being read.
- `DJILog::records` continues with an empty keychain when a log switches to a key set that
  wasn't provided, which decrypts everything after it to garbage. Here that's an error.

### Resume markers in older logs

Older logs (format version 11) carry a 4-byte marker, `39 30 00 00` (12345), between records
wherever recording resumed; newer ones only end with it. Upstream reads it as damaged data and
recovers with `utils::seek_to_next_record`, which scans to the 0xFF that ends a record but also
consumes the byte after it: the next record's type. Every record after that was then read one
byte late and failed too, often for hundreds of records, until the misreading happened to line
up again. 30 of 167 version-11 logs tested lost data this way.

Here `DJILog::records` skips the marker, and `seek_to_next_record` steps back over that byte.
All 167 then read without invalid data or failed records.

### Tips and warnings in older logs

In `src/record/mod.rs`, the text of tip and warning records (types 9, 10 and 24) is read with
the length `Record::Unknown` uses: in versions 7 to 12 the content is 2 bytes shorter than the
record length. Upstream read the full length, ran past the record's end, failed the end-byte
check, and fell back to `Unknown`, so in version 11 logs not one tip or warning decoded.
Versions 13 and up are unchanged; there the AES decoder pads short reads with zeros, which the
text mapping trims.

**Evidence.** In 9 version-11 logs, all 27 tips and warnings now decode to clean text ("Home
Point recorded. Return-to-Home Altitude:50.0 m"); before, none did. In 35 version-14 logs the
counts of these records are the same before and after (176, 677 and 18).

### Static battery information in version 14 logs

In `src/record/smart_battery_group.rs`, `SmartBatteryStatic` reads one more byte after `index`
in version 14 logs. Upstream's layout fits version 11 logs exactly, but version 14 logs have a
byte there, so upstream read every field after it one byte early: a 5,000 mAh battery's design
capacity came out as 1,280,000 (5,000 × 256), its 22 charge cycles as 5,632, and its 15.4 V as
3,942,400 mV. The byte is 0 in every log seen; what it means is unknown. Versions 12 and 13
are unchanged, as no logs of those versions were available to check.

**Evidence.** In version 11 logs the record is 41 bytes: its kind and exactly the 40 bytes of
upstream's layout. In version 14 logs it is 48 bytes, that layout plus the one byte, padded to
three 16-byte AES blocks. In 253 logs (9 of version 11, 244 of version 14, three models), every
record then reads a model's design capacity (5,000 mAh for every Mavic 3 Enterprise battery,
as specified), 15.4 V for the 4-cell batteries, and a charge cycle count that never
goes down from one flight of a battery to its next (34 batteries; 152 times up by exactly 1).
