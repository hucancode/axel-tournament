use anyhow::{Result, Context};
use async_trait::async_trait;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tokio::time::{timeout, Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File as TokioFile;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use surrealdb::sql::Thing;

use crate::players::Player;
use crate::sandbox::executor::{spawn_sandboxed, fd_to_file};
use crate::sandbox::cgroup::CgroupHandle;

/// BotPlayer runs player code in an isolated sandbox using Linux primitives
pub struct BotPlayer {
    id: Thing,
    pid: Option<Pid>,
    stdin: Arc<Mutex<TokioFile>>,
    stdout: Arc<Mutex<BufReader<TokioFile>>>,
    timeout_ms: u64,
    _cgroup: CgroupHandle, // Kept alive to enforce limits and cleanup on drop
}

impl BotPlayer {
    pub async fn new(player_id: Thing, binary_path: &str) -> Result<Self> {
        // Spawn the sandboxed process
        let process = tokio::task::spawn_blocking({
            let binary_path = binary_path.to_string();
            let player_id_str = player_id.to_string();
            move || {
                spawn_sandboxed(&player_id_str, Path::new(&binary_path))
            }
        })
        .await
        .context("Failed to spawn sandboxed process task")??;

        // Convert file descriptors to tokio Files
        let stdin_file = TokioFile::from_std(fd_to_file(process.stdin_fd));
        let stdout_file = TokioFile::from_std(fd_to_file(process.stdout_fd));
        let stdout_reader = BufReader::new(stdout_file);

        tracing::info!(
            player_id = %player_id,
            pid = %process.pid,
            binary = %binary_path,
            "Created sandboxed player"
        );

        Ok(Self {
            id: player_id,
            pid: Some(process.pid),
            stdin: Arc::new(Mutex::new(stdin_file)),
            stdout: Arc::new(Mutex::new(stdout_reader)),
            timeout_ms: 5000, // Default timeout
            _cgroup: process.cgroup, // Keep cgroup alive for resource limits and cleanup
        })
    }
}

#[async_trait]
impl Player for BotPlayer {
    async fn send_message(&self, message: &str) -> Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        let message_with_newline = format!("{}\n", message);
        stdin_guard
            .write_all(message_with_newline.as_bytes())
            .await
            .context("Failed to write to player stdin")?;
        stdin_guard
            .flush()
            .await
            .context("Failed to flush player stdin")?;

        tracing::debug!(player_id = %self.id, message = %message, "Sent message to player");
        Ok(())
    }

    async fn receive_message(&self) -> Result<String> {
        let timeout_duration = Duration::from_millis(self.timeout_ms);

        tracing::debug!(
            player_id = %self.id,
            timeout_ms = %self.timeout_ms,
            "Waiting for message from player"
        );

        let message = timeout(timeout_duration, async {
            let mut stdout_guard = self.stdout.lock().await;
            let mut line = String::new();
            stdout_guard
                .read_line(&mut line)
                .await
                .context("Failed to read from player stdout")?;

            let trimmed = line.trim().to_string();
            tracing::debug!(
                player_id = %self.id,
                message = %trimmed,
                "Received message from player"
            );
            Ok::<String, anyhow::Error>(trimmed)
        })
        .await
        .context(format!("Timeout waiting for message from player {}", self.id))??;

        Ok(message)
    }

    fn player_id(&self) -> &Thing {
        &self.id
    }

    async fn is_alive(&self) -> bool {
        if let Some(pid) = self.pid {
            // Check if process exists via kill with signal 0
            // Signal 0 doesn't actually send a signal, just checks if process exists
            kill(pid, None).is_ok()
        } else {
            false
        }
    }

    fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
}

impl Drop for BotPlayer {
    fn drop(&mut self) {
        if let Some(pid) = self.pid {
            tracing::debug!(player_id = %self.id, pid = %pid, "Cleaning up sandboxed player");

            // Send SIGTERM
            let _ = kill(pid, Signal::SIGTERM);

            // Wait briefly then SIGKILL
            std::thread::sleep(std::time::Duration::from_secs(1));
            let _ = kill(pid, Signal::SIGKILL);

            // Cgroup cleanup happens automatically via Drop
        }
    }
}
