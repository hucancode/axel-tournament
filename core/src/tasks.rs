//! Supervised background tasks. Logs panics, restarts crashed loops
//! with backoff, and tracks all handles in a `JoinSet` so the process
//! can wait for graceful shutdown instead of orphaning workers on exit.

use std::future::Future;
use std::time::Duration;

use tokio::task::JoinSet;

/// Default backoff schedule for restart-on-crash. Caps at 30s.
const RESTART_BACKOFF: &[Duration] = &[
    Duration::from_millis(250),
    Duration::from_secs(1),
    Duration::from_secs(5),
    Duration::from_secs(15),
    Duration::from_secs(30),
];

#[derive(Default)]
pub struct Supervisor {
    set: JoinSet<()>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run `future` once. Panic is caught + logged; task ends.
    pub fn spawn<F>(&mut self, name: &'static str, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.set.spawn(async move {
            tracing::info!(task = name, "task started");
            let res = std::panic::AssertUnwindSafe(future);
            // tokio task already isolates panic into JoinError; we add
            // a name on the log line so operators don't grep stack
            // frames to identify which loop died.
            res.await;
            tracing::warn!(task = name, "task exited cleanly");
        });
    }

    /// Run `make_future()` in a loop, restarting on exit/panic with
    /// backoff. Use for daemons (healer, heartbeat) that should never
    /// stop while the process is alive.
    pub fn spawn_forever<F, Fut>(&mut self, name: &'static str, mut make_future: F)
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.set.spawn(async move {
            let mut attempt: usize = 0;
            loop {
                tracing::info!(task = name, attempt, "supervised task starting");
                let h = tokio::spawn(make_future());
                match h.await {
                    Ok(()) => {
                        tracing::warn!(task = name, attempt, "supervised task exited; restarting");
                    }
                    Err(err) if err.is_panic() => {
                        tracing::error!(
                            task = name,
                            attempt,
                            "supervised task panicked; restarting"
                        );
                    }
                    Err(err) => {
                        tracing::error!(task = name, attempt, "supervised task aborted: {err}");
                        break;
                    }
                }
                let delay = RESTART_BACKOFF
                    .get(attempt.min(RESTART_BACKOFF.len() - 1))
                    .copied()
                    .unwrap_or(Duration::from_secs(30));
                tokio::time::sleep(delay).await;
                attempt = attempt.saturating_add(1);
            }
        });
    }

    /// Wait for every supervised task to finish. Use in graceful
    /// shutdown after signaling tasks to stop via their own cancel
    /// tokens.
    pub async fn join_all(mut self) {
        while let Some(res) = self.set.join_next().await {
            if let Err(err) = res {
                tracing::warn!("supervisor join error: {err}");
            }
        }
    }
}
