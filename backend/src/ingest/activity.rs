//! What log processing is doing right now, for the status admins see. Kept in
//! memory: it only describes the moment, and the processing loop runs in the
//! same process as the API.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// A step in processing one log.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Reading,
    RequestingKeys,
    Decoding,
    Saving,
}

#[derive(Clone, Serialize)]
pub struct Current {
    /// Relative to the flight log directory.
    pub path: String,
    pub stage: Stage,
    pub started_at: DateTime<Utc>,
}

/// Processing waits after a problem that may go away, such as DJI being unreachable.
#[derive(Clone, Serialize)]
pub struct Paused {
    pub reason: String,
    pub until: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    /// When the loop started on the current batch: the logs it has processed
    /// since it last found none pending. Progress counts from here.
    pub batch_started_at: Option<DateTime<Utc>>,
    /// The start of the batch finished last, so the status can show its
    /// result once nothing is pending.
    pub finished_batch_started_at: Option<DateTime<Utc>>,
    pub current: Option<Current>,
    pub paused: Option<Paused>,
}

/// Shared between the processing loop, which updates it, and the API.
#[derive(Clone, Default)]
pub struct Activity(Arc<Mutex<Snapshot>>);

impl Activity {
    pub fn snapshot(&self) -> Snapshot {
        self.lock().clone()
    }

    pub fn start(&self, path: &str) {
        let mut now = self.lock();
        now.batch_started_at.get_or_insert_with(Utc::now);
        now.current = Some(Current {
            path: path.to_owned(),
            stage: Stage::Reading,
            started_at: Utc::now(),
        });
        now.paused = None;
    }

    pub fn stage(&self, stage: Stage) {
        if let Some(current) = &mut self.lock().current {
            current.stage = stage;
        }
    }

    pub fn idle(&self) {
        let mut now = self.lock();
        *now = Snapshot {
            finished_batch_started_at: now.batch_started_at.or(now.finished_batch_started_at),
            ..Snapshot::default()
        };
    }

    /// A pause keeps the batch: the logs it waits for are still pending.
    pub fn pause(&self, reason: String, pause: Duration) {
        let mut now = self.lock();
        now.current = None;
        now.paused = Some(Paused {
            reason,
            until: Utc::now() + pause,
        });
    }

    fn lock(&self) -> MutexGuard<'_, Snapshot> {
        // Every update leaves a consistent snapshot, so one that panicked can be used.
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
