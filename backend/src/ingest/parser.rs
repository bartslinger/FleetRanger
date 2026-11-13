//! Runs the DJI log parser in a child process of its own, so that a log that
//! runs the parser out of memory, into a stack overflow or into an endless
//! loop ends only that process, not the backend (user's decisions,
//! 2026-11-13). The child is this binary with a hidden subcommand:
//!
//! - `key-request <file>` gives the request for the log's decryption keys,
//!   exactly as the parser writes it, so that stored answers keep matching.
//! - `decode <file>`, with DJI's answer to that request on stdin, gives the
//!   flight in the log.
//!
//! The child gets no environment, so no secrets, and uses no database or
//! network. It limits its own memory and processor time, and offers itself
//! first to the kernel when memory runs out. It writes its result as JSON on
//! stdout; a log it can't read is the log's fault, as is the child dying of
//! its own doing (aborting, crashing, its processor time running out). A
//! kill the backend didn't send, the time on the clock running out, or the
//! child not starting are errors that may pass: the log is tried again.

use std::io::{Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use dji_log_parser::DJILog;
use dji_log_parser::frame::records_to_frames;
use dji_log_parser::record::Record;
use dji_log_parser::record::smart_battery_group::SmartBatteryGroup;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use super::dji::{self, Keychains};
use super::flight::{self, BatteryInfo, Flight, Serials};

const KEY_REQUEST: &str = "key-request";
const DECODE: &str = "decode";

/// The child's address space. The largest test log, 8.2 MB, peaks at 60 MB
/// (2026-11-13): about 7 times its size, so 4 GB leaves ample room for an
/// upload of the largest size accepted (64 MB, so about 450 MB).
const MEMORY_LIMIT_MB: u64 = 4096;
/// The child's processor time. The largest test log takes 0.2 s.
const CPU_LIMIT_S: u64 = 60;
/// The time on the clock a child gets, longer than its processor time, as a
/// busy server may give it little of that. Both runs for a log stay under
/// the claim on it (`db::LEASE_MINUTES`), so no other backend takes it meanwhile.
const CLOCK_LIMIT: Duration = Duration::from_secs(90);
/// The child's exit code when it couldn't get to the log: the file not
/// readable, the limits not set. Not the log's fault.
const NOT_STARTED: i32 = 70;

/// The record types the parser knows (the `magic` values in its `record/mod.rs`).
/// A record of one of these types that comes out as `Unknown` failed to decode.
const KNOWN_RECORD_TYPES: [u8; 24] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 19, 22, 24, 25, 33, 40, 49, 50, 56, 62,
];

/// What a log contains, or why it can't give a flight (the log's fault).
pub type Processed = std::result::Result<Box<Decoded>, String>;

#[derive(Serialize, Deserialize)]
pub struct Decoded {
    /// `None` if the aircraft never took off.
    pub flight: Option<Flight>,
    /// The summary DJI's app wrote into the log, logged for comparison with ours.
    pub dji_total_time_s: f64,
    pub dji_total_distance_m: f32,
}

/// What `key-request` finds in a log.
#[derive(Serialize, Deserialize)]
pub enum KeyRequest {
    /// The log isn't encrypted (format versions before 13).
    None,
    /// The request to send DJI, exactly as the parser wrote it.
    Body(String),
}

/// The child's answer: its result, or why the log gives none.
#[derive(Serialize, Deserialize)]
enum Answer<T> {
    Done(T),
    Failed(String),
}

impl<T> From<std::result::Result<T, String>> for Answer<T> {
    fn from(result: std::result::Result<T, String>) -> Self {
        match result {
            Ok(value) => Answer::Done(value),
            Err(reason) => Answer::Failed(reason),
        }
    }
}

/// The request for a log's keys, if it needs any; an error if the child
/// couldn't run (it may pass).
pub async fn key_request(file: &Path) -> Result<std::result::Result<KeyRequest, String>> {
    run(KEY_REQUEST, file, "").await
}

/// The flight in a log, with DJI's answer to its key request if it is
/// encrypted; an error if the child couldn't run (it may pass).
pub async fn decode(file: &Path, dji_answer: Option<&str>) -> Result<Processed> {
    Ok(run(DECODE, file, dji_answer.unwrap_or_default())
        .await?
        .map(Box::new))
}

async fn run<T: DeserializeOwned>(
    command: &str,
    file: &Path,
    input: &str,
) -> Result<std::result::Result<T, String>> {
    let mut child = tokio::process::Command::new("/proc/self/exe")
        .arg(command)
        .arg(file)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // What the child says when it dies goes to the backend's log.
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .context("failed to start the log parser")?;
    let mut stdin = child.stdin.take().context("no input to the log parser")?;
    let output = tokio::time::timeout(CLOCK_LIMIT, async move {
        // A child that died before reading its input says why in how it ended.
        if let Err(error) = stdin.write_all(input.as_bytes()).await {
            tracing::debug!("the log parser took no input: {error}");
        }
        drop(stdin);
        child.wait_with_output().await
    })
    .await
    .map_err(|_| {
        anyhow!(
            "the log parser took longer than {} s",
            CLOCK_LIMIT.as_secs()
        )
    })?
    .context("failed to run the log parser")?;

    if let Some(died) = death(output.status) {
        return died.map(Err);
    }
    match serde_json::from_slice(&output.stdout) {
        Ok(Answer::Done(value)) => Ok(Ok(value)),
        Ok(Answer::Failed(reason)) => Ok(Err(reason)),
        Err(error) => Ok(Err(format!(
            "the log parser's answer is unreadable: {error}"
        ))),
    }
}

/// How a child that didn't finish its work ended: the log's fault (why), or
/// an error that may pass. None if it finished.
fn death(status: ExitStatus) -> Option<Result<String>> {
    if let Some(signal) = status.signal() {
        return Some(match signal {
            libc::SIGXCPU => Ok(format!(
                "the log parser needed more than {CPU_LIMIT_S} s of processor time"
            )),
            libc::SIGABRT => Ok(format!(
                "the log parser stopped, probably out of memory (limit {MEMORY_LIMIT_MB} MB)"
            )),
            libc::SIGSEGV | libc::SIGBUS | libc::SIGILL | libc::SIGFPE => {
                Ok(format!("the log parser crashed (signal {signal})"))
            }
            // Sent by something else, such as the kernel when the server
            // ran out of memory.
            _ => Err(anyhow!("the log parser was stopped (signal {signal})")),
        });
    }
    match status.code() {
        Some(0) => None,
        Some(NOT_STARTED) => Some(Err(anyhow!(
            "the log parser couldn't get to the log (see the log parser's message)"
        ))),
        Some(code) => Some(Ok(format!("the log parser stopped (exit code {code})"))),
        None => Some(Err(anyhow!("the log parser ended without an exit code"))),
    }
}

/// In the child: runs the subcommand `args` name, if they name one, and
/// returns its exit code.
pub fn run_child(args: &[String]) -> Option<i32> {
    let [command, file] = args else {
        return None;
    };
    if command != KEY_REQUEST && command != DECODE {
        return None;
    }
    if let Err(error) = limit() {
        eprintln!("log parser: failed to set its limits: {error}");
        return Some(NOT_STARTED);
    }
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("log parser: failed to read {file}: {error}");
            return Some(NOT_STARTED);
        }
    };
    let answer = if command == KEY_REQUEST {
        to_json(Answer::from(find_key_request(bytes)))
    } else {
        let mut dji_answer = String::new();
        if let Err(error) = std::io::stdin().read_to_string(&mut dji_answer) {
            eprintln!("log parser: failed to read DJI's answer: {error}");
            return Some(NOT_STARTED);
        }
        to_json(Answer::from(decode_log(bytes, &dji_answer)))
    };
    let written = answer.and_then(|json| {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(json.as_bytes())?;
        Ok(stdout.flush()?)
    });
    Some(match written {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("log parser: failed to answer: {error:#}");
            1
        }
    })
}

/// The answer as JSON, if it reads back the same: a value that isn't a
/// number, such as NaN, doesn't, and fails the log.
fn to_json<T: Serialize + DeserializeOwned>(answer: Answer<T>) -> Result<String> {
    let json = serde_json::to_string(&answer)?;
    if serde_json::from_str::<Answer<T>>(&json).is_ok() {
        return Ok(json);
    }
    Ok(serde_json::to_string(&Answer::<T>::Failed(
        "the log holds values that aren't numbers".to_owned(),
    ))?)
}

/// Limits the child's memory, processor time and core dumps, and makes it the first the
/// kernel stops when the server runs out of memory.
fn limit() -> std::io::Result<()> {
    if let Err(error) = std::fs::write("/proc/self/oom_score_adj", "1000") {
        eprintln!("log parser: failed to offer itself to the out-of-memory killer: {error}");
    }
    let memory = libc::rlimit {
        rlim_cur: MEMORY_LIMIT_MB * 1024 * 1024,
        rlim_max: MEMORY_LIMIT_MB * 1024 * 1024,
    };
    // Past the soft limit the kernel sends SIGXCPU, which ends the child;
    // the hard limit is only for a child that survives it.
    let cpu = libc::rlimit {
        rlim_cur: CPU_LIMIT_S,
        rlim_max: CPU_LIMIT_S + 5,
    };
    // A child that aborts writes no core dump, which would fill the disk.
    let no_core = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: setrlimit only reads the limit it is given, which outlives the call.
    let failed = unsafe {
        libc::setrlimit(libc::RLIMIT_AS, &memory) != 0
            || libc::setrlimit(libc::RLIMIT_CPU, &cpu) != 0
            || libc::setrlimit(libc::RLIMIT_CORE, &no_core) != 0
    };
    if failed {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn find_key_request(bytes: Vec<u8>) -> std::result::Result<KeyRequest, String> {
    let log =
        DJILog::from_bytes(bytes).map_err(|error| format!("not a readable DJI log: {error}"))?;
    // Logs from format version 13 on are encrypted.
    if log.version < 13 {
        return Ok(KeyRequest::None);
    }
    let request = log
        .keychains_request()
        .map_err(|error| format!("failed to read the log's key records: {error}"))?;
    serde_json::to_string(&request)
        .map(KeyRequest::Body)
        .map_err(|error| format!("failed to write the request for the log's keys: {error}"))
}

fn decode_log(bytes: Vec<u8>, dji_answer: &str) -> std::result::Result<Decoded, String> {
    let log =
        DJILog::from_bytes(bytes).map_err(|error| format!("not a readable DJI log: {error}"))?;
    let keychains = if log.version >= 13 {
        Some(keychains(dji_answer)?)
    } else {
        None
    };
    let records = log
        .records(keychains)
        .map_err(|error| format!("failed to decode the log: {error}"))?;

    let failed = records
        .iter()
        .filter(|record| match record {
            Record::Invalid(_) => true,
            Record::Unknown(record_type, _) => KNOWN_RECORD_TYPES.contains(record_type),
            _ => false,
        })
        .count();
    if failed > 0 {
        return Err(format!("{failed} records failed to decode"));
    }

    // The frames leave out the battery's static record.
    let battery_info = records
        .iter()
        .find_map(|record| match record {
            Record::SmartBatteryGroup(SmartBatteryGroup::SmartBatteryStatic(battery)) => {
                Some(BatteryInfo {
                    design_capacity_mah: i32::try_from(battery.designed_capacity)
                        .ok()
                        .filter(|&mah| mah > 0),
                    cycles: Some(i32::from(battery.loop_times)),
                })
            }
            _ => None,
        })
        .unwrap_or_default();
    let frames = records_to_frames(records, log.details.clone());
    Ok(Decoded {
        flight: flight::derive(&frames, Serials::from_details(&log.details), battery_info),
        dji_total_time_s: log.details.total_time,
        dji_total_distance_m: log.details.total_distance,
    })
}

/// The keys in DJI's answer, which the backend checked before passing it on.
fn keychains(dji_answer: &str) -> std::result::Result<Keychains, String> {
    match dji::keychains(dji_answer) {
        Ok(answer) => answer,
        Err(error) => Err(format!("{error:#}")),
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    use super::{NOT_STARTED, death};

    /// Only a child that died of its own doing fails the log (user's
    /// decision, 2026-11-13); being killed from outside, as by the kernel
    /// when the server runs out of memory, or not getting to the log, is
    /// tried again.
    #[test]
    fn only_deaths_of_its_own_doing_fail_the_log() {
        let from_signal = |signal| ExitStatus::from_raw(signal);
        let from_code = |code: i32| ExitStatus::from_raw(code << 8);
        assert!(death(from_code(0)).is_none());
        for status in [
            from_signal(libc::SIGABRT),
            from_signal(libc::SIGSEGV),
            from_signal(libc::SIGXCPU),
            from_code(1),
            from_code(101),
        ] {
            assert!(matches!(death(status), Some(Ok(_))), "{status:?}");
        }
        for status in [
            from_signal(libc::SIGKILL),
            from_signal(libc::SIGTERM),
            from_code(NOT_STARTED),
        ] {
            assert!(matches!(death(status), Some(Err(_))), "{status:?}");
        }
    }
}
