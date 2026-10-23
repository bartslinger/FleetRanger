//! Server-sent events for the web app's header: log processing, for admins,
//! sent as it changes instead of being polled.
//!
//! Whatever changes it calls [`Changes::notify`]. Each stream then works out
//! the current state for its user and sends what differs from what it sent
//! before. A refresh now and then catches what nothing signals, such as logs
//! imported from the command line.

use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::{FromRef, State},
    http::{HeaderName, HeaderValue},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use futures_util::stream;
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::watch;

use crate::auth::Actor;
use crate::error::ApiError;
use crate::ingest::{activity::Activity, service as ingest};
use crate::policy::Permission;

/// Changes in quick succession (each step of processing a log) go out as one.
const SETTLE: Duration = Duration::from_millis(250);
const REFRESH: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Default)]
struct Signal {
    version: u64,
    closed: bool,
}

/// Tells the open streams that something they show may have changed.
#[derive(Clone)]
pub struct Changes(Arc<watch::Sender<Signal>>);

impl Default for Changes {
    fn default() -> Self {
        Self(Arc::new(watch::channel(Signal::default()).0))
    }
}

impl Changes {
    pub fn notify(&self) {
        self.0.send_modify(|signal| signal.version += 1);
    }

    /// Ends every stream, so the server can shut down: it waits for open
    /// connections, and a stream would otherwise stay open.
    pub fn close(&self) {
        self.0.send_modify(|signal| signal.closed = true);
    }
}

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    Activity: FromRef<S>,
    Changes: FromRef<S>,
{
    Router::new().route("/api/events", get(events))
}

struct Stream {
    db: PgPool,
    actor: Actor,
    activity: Activity,
    signal: watch::Receiver<Signal>,
    started: bool,
    queue: VecDeque<Event>,
    /// What was sent last, per event, as JSON.
    processing: Option<String>,
}

async fn events(
    actor: Actor,
    State(db): State<PgPool>,
    State(activity): State<Activity>,
    State(Changes(changes)): State<Changes>,
) -> impl IntoResponse {
    let mut signal = changes.subscribe();
    signal.borrow_and_update();
    let state = Stream {
        db,
        actor,
        activity,
        signal,
        started: false,
        queue: VecDeque::new(),
        processing: None,
    };
    let events = stream::unfold(state, |mut state| async move {
        let event = state.next().await?;
        Some((Ok::<_, Infallible>(event), state))
    });
    (
        // nginx would otherwise hold events back in its buffer.
        [(
            HeaderName::from_static("x-accel-buffering"),
            HeaderValue::from_static("no"),
        )],
        Sse::new(events).keep_alive(KeepAlive::default()),
    )
}

impl Stream {
    /// The next event to send; None ends the stream.
    async fn next(&mut self) -> Option<Event> {
        loop {
            if self.signal.borrow().closed {
                return None;
            }
            if let Some(event) = self.queue.pop_front() {
                return Some(event);
            }
            if self.started {
                tokio::select! {
                    changed = self.signal.changed() => {
                        if changed.is_err() {
                            return None;
                        }
                        tokio::time::sleep(SETTLE).await;
                        self.signal.borrow_and_update();
                    }
                    () = tokio::time::sleep(REFRESH) => {}
                }
            }
            self.started = true;
            self.refresh().await;
        }
    }

    /// Queues the states that differ from what was sent.
    async fn refresh(&mut self) {
        if self.actor.grants.may(Permission::LogsManage, None) {
            match ingest::status(&self.db, &self.actor, &self.activity).await {
                Ok(status) => queue(&mut self.queue, &mut self.processing, "processing", &status),
                Err(error) => failed("processing status", error),
            }
        }
    }
}

/// Logged; the stream goes on and tries again at the next change.
fn failed(what: &str, error: ApiError) {
    if let ApiError::Internal(error) = error {
        tracing::warn!(error = format!("{error:#}"), "{what} for events failed");
    }
}

fn queue(
    queue: &mut VecDeque<Event>,
    sent: &mut Option<String>,
    name: &str,
    value: &impl Serialize,
) {
    let Ok(json) = serde_json::to_string(value) else {
        return;
    };
    if sent.as_deref() != Some(json.as_str()) {
        queue.push_back(Event::default().event(name).data(&json));
        *sent = Some(json);
    }
}
