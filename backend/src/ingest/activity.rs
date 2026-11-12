//! What log processing is doing right now, for the status admins see. Kept in
//! memory: it only describes the moment, and the processing loop runs in the
//! same process as the API.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::events::Changes;

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
    /// Whose log it is: only that organisation sees it.
    #[serde(skip)]
    pub organisation_id: Uuid,
    /// Relative to the flight log directory.
    pub path: String,
    pub stage: Stage,
    pub started_at: DateTime<Utc>,
}

/// Processing waits after a problem that may go away, such as DJI being
/// unreachable. Every organisation's admins see it: the reason names none.
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

/// Shared between the processing loop, which updates it, and the API. Every
/// update tells the open event streams.
#[derive(Clone)]
pub struct Activity {
    now: Arc<Mutex<Snapshot>>,
    changes: Changes,
}

impl Activity {
    pub fn new(changes: Changes) -> Self {
        Self {
            now: Arc::default(),
            changes,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.lock().clone()
    }

    pub fn start(&self, organisation_id: Uuid, path: &str) {
        let mut now = self.lock();
        now.batch_started_at.get_or_insert_with(Utc::now);
        now.current = Some(Current {
            organisation_id,
            path: path.to_owned(),
            stage: Stage::Reading,
            started_at: Utc::now(),
        });
        now.paused = None;
        self.changes.notify();
    }

    pub fn stage(&self, stage: Stage) {
        if let Some(current) = &mut self.lock().current {
            current.stage = stage;
        }
        self.changes.notify();
    }

    /// Done with the current log, whatever became of it.
    pub fn finished(&self) {
        self.lock().current = None;
        self.changes.notify();
    }

    /// Nothing due.
    pub fn idle(&self) {
        let mut now = self.lock();
        *now = Snapshot {
            finished_batch_started_at: now.batch_started_at.or(now.finished_batch_started_at),
            ..Snapshot::default()
        };
        self.changes.notify();
    }

    /// Trouble outside any log, such as the database being unreachable. The
    /// details, which may name the servers, are only in the server log. A
    /// pause keeps the batch: the logs it waits for are still pending.
    pub fn pause(&self, pause: Duration) {
        let mut now = self.lock();
        now.current = None;
        now.paused = Some(Paused {
            reason: "A problem outside the logs; the server log has the details".to_owned(),
            until: Utc::now() + pause,
        });
        self.changes.notify();
    }

    fn lock(&self) -> MutexGuard<'_, Snapshot> {
        // Every update leaves a consistent snapshot, so one that panicked can be used.
        self.now
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
