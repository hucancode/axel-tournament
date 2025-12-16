use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions,
};
use bollard::image::BuildImageOptions;
use bollard::models::HostConfig;
use bollard::Docker;
use chrono::Utc;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::api_client::{Match, MatchResult, ParticipantResult};

pub struct DockerRunner {
    docker: Docker,
}

impl DockerRunner {
    pub fn new(docker: Docker) -> Self {
        Self { docker }
    }

    pub async fn execute_match(&self, match_data: &Match) -> Result<MatchResult> {
        let started_at = Utc::now();

        // 1. Get game Docker image (build if needed)
        let image_tag = self.ensure_game_image(&match_data.game_id).await?;

        // 2. Prepare submission files
        let work_dir = self.prepare_workspace(match_data).await?;

        // 3. Run match in container
        let results = self.run_match_container(&image_tag, &work_dir, match_data).await?;

        // 4. Cleanup
        self.cleanup_workspace(&work_dir).await?;

        let completed_at = Utc::now();

        Ok(MatchResult {
            status: "completed".to_string(),
            participants: results,
            metadata: serde_json::json!({
                "execution_time_ms": (completed_at - started_at).num_milliseconds()
            }),
            started_at: started_at.to_rfc3339(),
            completed_at: completed_at.to_rfc3339(),
        })
    }

    async fn ensure_game_image(&self, game_id: &str) -> Result<String> {
        let image_tag = format!("axel-game-{}", game_id);

        // Check if image exists
        if self.docker.inspect_image(&image_tag).await.is_ok() {
            tracing::info!("Using cached Docker image: {}", image_tag);
            return Ok(image_tag);
        }

        tracing::info!("Building Docker image for game {}", game_id);

        // Build image from Dockerfile
        let dockerfile_path = format!("dockerfiles/game_{}.dockerfile", game_id);
        if !Path::new(&dockerfile_path).exists() {
            anyhow::bail!("Dockerfile not found for game {}: {}", game_id, dockerfile_path);
        }

        // Read Dockerfile content
        let dockerfile_content = fs::read_to_string(&dockerfile_path).await?;

        // Create a tar archive with the Dockerfile
        let mut tar = tar::Builder::new(Vec::new());
        let dockerfile_bytes = dockerfile_content.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_size(dockerfile_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "Dockerfile", dockerfile_bytes)?;
        let tar_bytes = tar.into_inner()?;

        // Build image
        let build_options = BuildImageOptions {
            t: image_tag.clone(),
            rm: true,
            ..Default::default()
        };

        let mut stream = self.docker.build_image(build_options, None, Some(tar_bytes.into()));

        // Wait for build to complete and log output
        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(stream) = output.stream {
                        tracing::debug!("Docker build: {}", stream.trim());
                    }
                    if let Some(error) = output.error {
                        anyhow::bail!("Docker build error: {}", error);
                    }
                }
                Err(e) => anyhow::bail!("Docker build failed: {}", e),
            }
        }

        tracing::info!("Successfully built Docker image: {}", image_tag);
        Ok(image_tag)
    }

    async fn prepare_workspace(&self, match_data: &Match) -> Result<String> {
        let workspace_dir = format!("/tmp/match_{}", match_data.id.replace(':', "_"));
        fs::create_dir_all(&workspace_dir).await?;

        // Copy submission files to workspace
        for (idx, participant) in match_data.participants.iter().enumerate() {
            let submission_id = participant.submission_id.replace(':', "_");
            // Look for submission file in uploads directory
            let possible_paths = vec![
                format!("uploads/{}", submission_id),
                format!("uploads/submission_{}", submission_id),
            ];

            let mut found = false;
            for src_path in possible_paths {
                if Path::new(&src_path).exists() {
                    let dest_file = format!("{}/player_{}.code", workspace_dir, idx);
                    fs::copy(&src_path, &dest_file)
                        .await
                        .context(format!("Failed to copy submission {}", participant.submission_id))?;
                    found = true;
                    break;
                }
            }

            if !found {
                anyhow::bail!("Submission file not found for {}", participant.submission_id);
            }
        }

        Ok(workspace_dir)
    }

    async fn run_match_container(
        &self,
        image_tag: &str,
        work_dir: &str,
        match_data: &Match,
    ) -> Result<Vec<ParticipantResult>> {
        // Create container with resource limits
        let config = Config {
            image: Some(image_tag.to_string()),
            host_config: Some(HostConfig {
                binds: Some(vec![format!("{}:/workspace:ro", work_dir)]),
                memory: Some(512 * 1024 * 1024),      // 512MB
                nano_cpus: Some(1_000_000_000),       // 1 CPU
                network_mode: Some("none".to_string()), // No network access
                ..Default::default()
            }),
            working_dir: Some("/workspace".to_string()),
            ..Default::default()
        };

        let container_name = format!("match-{}", match_data.id.replace(':', "_"));
        let create_options = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let container = self
            .docker
            .create_container(Some(create_options), config)
            .await?;

        // Start container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;

        // Wait for completion (with timeout)
        let timeout = std::time::Duration::from_secs(300); // 5 minutes
        let wait_result = tokio::time::timeout(
            timeout,
            async {
                let mut stream = self.docker.wait_container(
                    &container.id,
                    None::<WaitContainerOptions<String>>,
                );
                while let Some(_) = stream.next().await {}
                Ok::<(), anyhow::Error>(())
            },
        )
        .await;

        if wait_result.is_err() {
            // Timeout - stop and remove container
            let _ = self.docker.stop_container(&container.id, None).await;
            let _ = self
                .docker
                .remove_container(
                    &container.id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await;
            anyhow::bail!("Match execution timed out after 5 minutes");
        }

        // Get container logs (stdout contains match results in JSON)
        let mut logs_stream = self.docker.logs(
            &container.id,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        );

        let mut logs = String::new();
        while let Some(log) = logs_stream.next().await {
            if let Ok(log_output) = log {
                logs.push_str(&log_output.to_string());
            }
        }

        // Remove container
        self.docker
            .remove_container(
                &container.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        // Parse results from logs
        let results = self.parse_match_results(&logs, match_data)?;

        Ok(results)
    }

    fn parse_match_results(
        &self,
        logs: &str,
        match_data: &Match,
    ) -> Result<Vec<ParticipantResult>> {
        // Expected format: JSON with scores for each submission_id
        // Try to find JSON in logs (may be mixed with other output)
        let json_start = logs.find('{');
        let json_end = logs.rfind('}');

        let json_str = if let (Some(start), Some(end)) = (json_start, json_end) {
            &logs[start..=end]
        } else {
            logs
        };

        let scores: HashMap<String, f64> = serde_json::from_str(json_str)
            .context("Failed to parse match results from container output")?;

        let mut results: Vec<ParticipantResult> = match_data
            .participants
            .iter()
            .map(|p| {
                let score = scores.get(&p.submission_id).copied().unwrap_or(0.0);
                ParticipantResult {
                    submission_id: p.submission_id.clone(),
                    score,
                    rank: None,
                    is_winner: false,
                    metadata: None,
                }
            })
            .collect();

        // Determine rankings
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        for (idx, result) in results.iter_mut().enumerate() {
            result.rank = Some((idx + 1) as u32);
            result.is_winner = idx == 0;
        }

        Ok(results)
    }

    async fn cleanup_workspace(&self, work_dir: &str) -> Result<()> {
        fs::remove_dir_all(work_dir).await?;
        Ok(())
    }
}
