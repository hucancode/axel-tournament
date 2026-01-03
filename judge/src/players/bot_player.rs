use anyhow::{Result, Context};
use async_trait::async_trait;
use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, AttachContainerOptions, LogOutput};
use bollard::Docker;
use futures_util::StreamExt;
use tokio::time::{timeout, Duration};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::player::Player;

/// BotPlayer spins up a Docker instance and forwards messages
/// Forwards messages from GameLogic to stdin, forwards messages from stdout to GameLogic
pub struct BotPlayer {
    container_id: String,
    docker: Docker,
    attach_stream: Arc<Mutex<Option<bollard::container::AttachContainerResults>>>,
    timeout_ms: u64,
}

impl BotPlayer {
    pub async fn new(player_id: String, binary_path: &str) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;

        let sandbox_image = std::env::var("SANDBOX_IMAGE")
            .unwrap_or_else(|_| "debian:trixie-slim".to_string());

        // Create container configuration
        let config = Config {
            image: Some(sandbox_image),
            cmd: Some(vec!["/player".to_string()]),
            host_config: Some(bollard::models::HostConfig {
                memory: Some(64 * 1024 * 1024), // 64MB
                cpu_quota: Some(100000), // 1.0 CPU
                network_mode: Some("none".to_string()),
                mounts: Some(vec![bollard::models::Mount {
                    target: Some("/player".to_string()),
                    source: Some(binary_path.to_string()),
                    typ: Some(bollard::models::MountTypeEnum::BIND),
                    read_only: Some(true),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            open_stdin: Some(true),
            stdin_once: Some(false),
            tty: Some(false),
            ..Default::default()
        };

        let container_name = format!("player-{}-{}", player_id, uuid::Uuid::new_v4());
        let create_options = Some(CreateContainerOptions {
            name: container_name.as_str(),
            ..Default::default()
        });

        let container = docker.create_container(create_options, config).await?;
        let container_id = container.id;

        // Start container
        docker.start_container(&container_id, None::<StartContainerOptions<String>>).await?;

        // Attach to container for stdin/stdout communication
        let attach_options: AttachContainerOptions<String> = AttachContainerOptions {
            stdin: Some(true),
            stdout: Some(true),
            stderr: Some(false),
            stream: Some(true),
            logs: Some(false),
            detach_keys: None,
        };

        let attach_stream = docker.attach_container(&container_id, Some(attach_options)).await?;

        tracing::info!("Created Docker container {} for bot player {}", container_id, player_id);

        Ok(Self {
            container_id,
            docker,
            attach_stream: Arc::new(Mutex::new(Some(attach_stream))),
            timeout_ms: 5000, // Default timeout
        })
    }
}

#[async_trait]
impl Player for BotPlayer {
    async fn send_message(&self, message: &str) -> Result<()> {
        let mut stream_guard = self.attach_stream.lock().await;
        if let Some(ref mut stream) = *stream_guard {
            let message_with_newline = format!("{}\n", message);
            stream.input.write_all(message_with_newline.as_bytes()).await
                .context("Failed to send message to container stdin")?;
            stream.input.flush().await
                .context("Failed to flush container stdin")?;

            tracing::debug!("Sent message to container: {}", message);
        }
        Ok(())
    }

    async fn receive_message(&self) -> Result<String> {
        let mut stream_guard = self.attach_stream.lock().await;
        if let Some(ref mut stream) = *stream_guard {
            tracing::debug!("Waiting for move from container {} (timeout: {}ms)", self.container_id, self.timeout_ms);

            let move_str = timeout(Duration::from_millis(self.timeout_ms), async {
                while let Some(output_result) = stream.output.next().await {
                    match output_result {
                        Ok(LogOutput::StdOut { message }) => {
                            let line = String::from_utf8_lossy(&message).trim().to_string();
                            tracing::debug!("Received stdout from container {}: '{}'", self.container_id, line);
                            if !line.is_empty() {
                                return line;
                            }
                        }
                        Ok(LogOutput::StdErr { message }) => {
                            let stderr_msg = String::from_utf8_lossy(&message);
                            tracing::warn!("Container {} stderr: {}", self.container_id, stderr_msg.trim());
                        }
                        Ok(LogOutput::Console { message }) => {
                            let console_msg = String::from_utf8_lossy(&message);
                            tracing::debug!("Container {} console: {}", self.container_id, console_msg.trim());
                        }
                        Err(e) => {
                            tracing::error!("Docker stream error for container {}: {}", self.container_id, e);
                            return String::new();
                        }
                        _ => {
                            tracing::trace!("Container {} other log output received", self.container_id);
                        }
                    }
                }
                tracing::warn!("No more output from container {}", self.container_id);
                String::new()
            })
            .await
            .context(format!("Timeout waiting for move from container {}", self.container_id))?;

            tracing::debug!("Final move from container {}: '{}'", self.container_id, move_str);
            Ok(move_str)
        } else {
            anyhow::bail!("No attach stream available for container {}", self.container_id)
        }
    }

    fn player_id(&self) -> &str {
        &self.container_id
    }

    async fn is_alive(&self) -> bool {
        match self.docker.inspect_container(&self.container_id, None).await {
            Ok(details) => {
                if let Some(state) = details.state {
                    let running = state.running.unwrap_or(false);
                    let exit_code = state.exit_code.unwrap_or(0);

                    if !running && exit_code != 0 {
                        tracing::warn!("Container {} exited with code {}", self.container_id, exit_code);
                    }

                    running
                } else {
                    false
                }
            }
            Err(e) => {
                tracing::error!("Failed to inspect container {}: {}", self.container_id, e);
                false
            }
        }
    }

    fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
}

impl Drop for BotPlayer {
    fn drop(&mut self) {
        let docker = self.docker.clone();
        let container_id = self.container_id.clone();
        tokio::spawn(async move {
            let options = RemoveContainerOptions {
                force: true,
                ..Default::default()
            };
            let _ = docker.remove_container(&container_id, Some(options)).await;
            tracing::debug!("Cleaned up container {} on drop", container_id);
        });
    }
}
