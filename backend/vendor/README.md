# Vendored dependencies

## dji-log-parser

The published [dji-log-parser 0.5.7](https://crates.io/crates/dji-log-parser/0.5.7), the latest
release, as unpacked from the `.crate` file (SHA-256
`13d15112d03932c3a4122c43db9604ec4378a41621193b29fb8f8b2e8502ba0b`). The upstream commit is in
`.cargo_vcs_info.json`. The crate ships without its license file, so `LICENSE` was copied from
that commit.

It is vendored so it can be patched. Each patch has its own section below, with the reason for
it. Drop a patch once upstream fixes the problem.

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
