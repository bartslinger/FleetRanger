//! Writes the install's address into the controller app's APK, so the app
//! knows where to connect without anyone typing it.
//!
//! The address goes into the APK Signing Block, which sits between the ZIP
//! entries and the central directory. Android's verifier ignores ID-value
//! pairs it doesn't know, so the signature stays valid; app stores tag
//! channels the same way. Layout of the block:
//!
//! ```text
//! u64 size (of the rest)  |  pairs: u64 length, u32 id, value  |  u64 size  |  "APK Sig Block 42"
//! ```
//!
//! The app reads the pair back from its own installed APK (`Setup.kt`).

use anyhow::{Context, Result, bail, ensure};

/// Our pair's ID ("FRG1"); not one of Android's or the known stores'.
pub const SERVER_ID: u32 = 0x4652_4731;
/// The verity padding pair, which keeps the block a multiple of 4096 bytes.
const PADDING_ID: u32 = 0x4272_6577;
const MAGIC: &[u8; 16] = b"APK Sig Block 42";
const EOCD_SIGNATURE: u32 = 0x0605_4b50;
const EOCD_SIZE: usize = 22;

/// A copy of `apk` whose signing block also holds `server`, replacing an
/// earlier one.
pub fn with_server(apk: &[u8], server: &str) -> Result<Vec<u8>> {
    let eocd = find_eocd(apk)?;
    let cd_offset = read_u32(apk, eocd + 16)? as usize;
    ensure!(
        cd_offset <= eocd,
        "the central directory starts after its end"
    );
    let (block_start, pairs) = signing_block(apk, cd_offset)?;

    let had_padding = pairs.iter().any(|(id, _)| *id == PADDING_ID);
    let mut kept: Vec<(u32, &[u8])> = pairs
        .into_iter()
        .filter(|(id, _)| *id != PADDING_ID && *id != SERVER_ID)
        .collect();
    kept.push((SERVER_ID, server.as_bytes()));

    let mut body = Vec::new();
    for (id, value) in &kept {
        body.extend_from_slice(&(value.len() as u64 + 4).to_le_bytes());
        body.extend_from_slice(&id.to_le_bytes());
        body.extend_from_slice(value);
    }
    // Both sizes and the magic around the pairs: 8 + pairs + 8 + 16.
    if had_padding {
        let unpadded = body.len() + 32;
        let mut padding = (4096 - unpadded % 4096) % 4096;
        // A pair takes at least 12 bytes (length and ID).
        if padding != 0 && padding < 12 {
            padding += 4096;
        }
        if padding != 0 {
            body.extend_from_slice(&(padding as u64 - 8).to_le_bytes());
            body.extend_from_slice(&PADDING_ID.to_le_bytes());
            body.resize(body.len() + padding - 12, 0);
        }
    }
    let size = (body.len() + 24) as u64;

    let mut out = Vec::with_capacity(apk.len() + body.len());
    out.extend_from_slice(&apk[..block_start]);
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&body);
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(MAGIC);
    let new_cd_offset = out.len();
    out.extend_from_slice(&apk[cd_offset..]);
    // The end of central directory record points at the moved directory.
    let eocd_out = eocd - cd_offset + new_cd_offset;
    let offset = u32::try_from(new_cd_offset).context("the APK is too large")?;
    out[eocd_out + 16..eocd_out + 20].copy_from_slice(&offset.to_le_bytes());
    Ok(out)
}

/// The end of central directory record: the last one, as the comment after
/// it may be up to 65535 bytes long.
fn find_eocd(apk: &[u8]) -> Result<usize> {
    ensure!(apk.len() >= EOCD_SIZE, "too short for a ZIP file");
    let lowest = apk.len().saturating_sub(EOCD_SIZE + 0xffff);
    for start in (lowest..=apk.len() - EOCD_SIZE).rev() {
        let comment_length = read_u16(apk, start + 20)? as usize;
        if read_u32(apk, start)? == EOCD_SIGNATURE
            && start + EOCD_SIZE + comment_length == apk.len()
        {
            return Ok(start);
        }
    }
    bail!("not a ZIP file: no end of central directory")
}

/// The signing block's ID-value pairs.
type Pairs<'a> = Vec<(u32, &'a [u8])>;

/// Where the signing block starts, and its pairs.
fn signing_block(apk: &[u8], cd_offset: usize) -> Result<(usize, Pairs<'_>)> {
    ensure!(cd_offset >= 32, "no APK Signing Block");
    ensure!(
        &apk[cd_offset - 16..cd_offset] == MAGIC,
        "no APK Signing Block: not signed with v2 or later"
    );
    let size = read_u64(apk, cd_offset - 24)? as usize;
    let start = cd_offset
        .checked_sub(size + 8)
        .context("the APK Signing Block is larger than the file")?;
    ensure!(
        read_u64(apk, start)? as usize == size,
        "the APK Signing Block's sizes differ"
    );

    let mut pairs = Vec::new();
    let mut at = start + 8;
    let end = cd_offset - 24;
    while at < end {
        let length = read_u64(apk, at)? as usize;
        ensure!(
            length >= 4 && at + 8 + length <= end,
            "a broken pair in the APK Signing Block"
        );
        let id = read_u32(apk, at + 8)?;
        pairs.push((id, &apk[at + 12..at + 8 + length]));
        at += 8 + length;
    }
    Ok((start, pairs))
}

fn read_u16(bytes: &[u8], at: usize) -> Result<u16> {
    let slice = bytes.get(at..at + 2).context("past the end of the APK")?;
    Ok(u16::from_le_bytes(slice.try_into()?))
}

fn read_u32(bytes: &[u8], at: usize) -> Result<u32> {
    let slice = bytes.get(at..at + 4).context("past the end of the APK")?;
    Ok(u32::from_le_bytes(slice.try_into()?))
}

fn read_u64(bytes: &[u8], at: usize) -> Result<u64> {
    let slice = bytes.get(at..at + 8).context("past the end of the APK")?;
    Ok(u64::from_le_bytes(slice.try_into()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ZIP-shaped APK: entries, a signing block with a signature pair and
    /// padding to 4096, a central directory, and the end record.
    fn apk() -> Vec<u8> {
        let mut apk = b"local file entries".to_vec();
        let signature = (0x7109_871a_u32, vec![7u8; 100]);
        let mut body = Vec::new();
        body.extend_from_slice(&(signature.1.len() as u64 + 4).to_le_bytes());
        body.extend_from_slice(&signature.0.to_le_bytes());
        body.extend_from_slice(&signature.1);
        let padding = 4096 - (body.len() + 32);
        body.extend_from_slice(&(padding as u64 - 8).to_le_bytes());
        body.extend_from_slice(&PADDING_ID.to_le_bytes());
        body.resize(body.len() + padding - 12, 0);
        let size = (body.len() + 24) as u64;
        apk.extend_from_slice(&size.to_le_bytes());
        apk.extend_from_slice(&body);
        apk.extend_from_slice(&size.to_le_bytes());
        apk.extend_from_slice(MAGIC);
        let cd_offset = apk.len() as u32;
        apk.extend_from_slice(b"central directory");
        apk.extend_from_slice(&EOCD_SIGNATURE.to_le_bytes());
        apk.extend_from_slice(&[0; 12]);
        apk.extend_from_slice(&cd_offset.to_le_bytes());
        apk.extend_from_slice(&[0; 2]);
        apk
    }

    fn server_in(apk: &[u8]) -> (usize, Option<String>, Vec<u32>) {
        let eocd = find_eocd(apk).unwrap();
        let cd_offset = read_u32(apk, eocd + 16).unwrap() as usize;
        let (start, pairs) = signing_block(apk, cd_offset).unwrap();
        let server = pairs
            .iter()
            .find(|(id, _)| *id == SERVER_ID)
            .map(|(_, value)| String::from_utf8(value.to_vec()).unwrap());
        (
            cd_offset - start,
            server,
            pairs.iter().map(|(id, _)| *id).collect(),
        )
    }

    // The signature pair and the central directory survive, the end record
    // points at the moved directory, and the block stays 4096-aligned.
    #[test]
    fn adds_the_server_and_keeps_the_signature() {
        let original = apk();
        let tagged = with_server(&original, "https://fleet.example.org").unwrap();
        let (block_size, server, ids) = server_in(&tagged);
        assert_eq!(server.as_deref(), Some("https://fleet.example.org"));
        assert_eq!(ids, [0x7109_871a, SERVER_ID, PADDING_ID]);
        assert_eq!(block_size % 4096, 0);
        let cd_offset = read_u32(&tagged, find_eocd(&tagged).unwrap() + 16).unwrap() as usize;
        assert!(tagged[cd_offset..].starts_with(b"central directory"));
        assert_eq!(&tagged[..18], b"local file entries");
    }
}
