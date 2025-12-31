use anyhow::Result;
use bollard::Docker;
use bollard::container::{Config, AttachContainerOptions, RemoveContainerOptions};
use bollard::models::HostConfig;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{timeout, Duration};

pub struct DockerPlayer {
    docker: Docker,
    container_id: String,
    stdin: tokio::io::DuplexStream,
    stdout: BufReader<tokio::io::DuplexStream>,
}

impl DockerPlayer {
    pub async fn new(binary_path: &str, memory_mb: i64, cpu_cores: f64) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;

        let config = Config {
            image: Some("debian:bookworm-slim".to_string()),
            cmd: Some(vec![binary_path.to_string()]),
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(false),
            open_stdin: Some(true),
            stdin_once: Some(false),
            tty: Some(false),
            host_config: Some(HostConfig {
                network_mode: Some("none".to_string()),
                memory: Some(memory_mb * 1024 * 1024),
                nano_cpus: Some((cpu_cores * 1_000_000_000.0) as i64),
                pids_limit: Some(64),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container = docker
            .create_container::<String, String>(None, config)
            .await?;
        let container_id = container.id;

        docker.start_container::<String>(&container_id, None).await?;

        let attach_options = AttachContainerOptions::<String> {
            stdin: Some(true),
            stdout: Some(true),
            stderr: Some(false),
            stream: Some(true),
            ..Default::default()
        };

        let _attach_result = docker.attach_container(&container_id, Some(attach_options)).await?;

        let (stdin_stream, stdout_stream) = tokio::io::duplex(8192);
        let stdout_reader = BufReader::new(stdout_stream);

        Ok(Self {
            docker,
            container_id,
            stdin: stdin_stream,
            stdout: stdout_reader,
        })
    }

    pub async fn send(&mut self, message: &str) -> Result<()> {
        self.stdin.write_all(message.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn read_with_timeout(&mut self, timeout_duration: Duration) -> Result<String> {
        let mut line = String::new();
        match timeout(timeout_duration, self.stdout.read_line(&mut line)).await {
            Ok(Ok(0)) => Err(anyhow::anyhow!("Player disconnected")),
            Ok(Ok(_)) => Ok(line.trim().to_string()),
            Ok(Err(e)) => Err(anyhow::anyhow!("Read error: {}", e)),
            Err(_) => Err(anyhow::anyhow!("TLE")),
        }
    }

    pub async fn cleanup(self) {
        let _ = self.docker.remove_container(
            &self.container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        ).await;
    }
}
