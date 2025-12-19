use anyhow::{Context, Result};
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    BuildImageOptions, BuildImageOptionsBuilder, CreateContainerOptions,
    CreateContainerOptionsBuilder, LogsOptionsBuilder, RemoveContainerOptionsBuilder,
    StartContainerOptions, StopContainerOptions, WaitContainerOptions,
};
use bollard::Docker;
use bytes::Bytes;
use chrono::Utc;
use futures_util::StreamExt;
use http_body_util::{Either, Full};
use rand::{rng, Rng};
use std::collections::HashMap;
use tokio::fs;

use crate::db_client::{DbClient, Game, Match, MatchResult, ParticipantResult};

pub struct DockerRunner {
    docker: Docker,
    db_client: DbClient,
}

impl DockerRunner {
    pub fn new(docker: Docker, db_client: DbClient) -> Self {
        Self { docker, db_client }
    }

    pub async fn execute_match(&self, match_data: &Match) -> Result<MatchResult> {
        let started_at = Utc::now();
        let rounds: u32 = rng().random_range(100..=120);

        // 0. Fetch game details from API
        let game = self
            .db_client
            .fetch_game(&match_data.game_id)
            .await
            .context(format!("Failed to fetch game {}", match_data.game_id))?;

        // 1. Use universal Docker image
        let image_tag = "axel-sandbox".to_string();

        // 2. Prepare submission files
        let work_dir = self.prepare_workspace(match_data, &game).await?;

        // 3. Run match in container
        let results = self
            .run_match_container(&image_tag, &work_dir, match_data, rounds)
            .await?;

        // 4. Cleanup
        self.cleanup_workspace(&work_dir).await?;

        let completed_at = Utc::now();

        Ok(MatchResult {
            participants: results,
            metadata: serde_json::json!({
                "execution_time_ms": (completed_at - started_at).num_milliseconds(),
                "rounds": rounds
            }),
            started_at,
            completed_at,
        })
    }

    pub async fn ensure_docker_image(&self) -> Result<()> {
        let image_tag = "axel-sandbox";

        // Check if image exists
        if self.docker.inspect_image(image_tag).await.is_ok() {
            tracing::info!("Universal Docker image already exists: {}", image_tag);
            return Ok(());
        }

        tracing::info!("Building universal Docker image...");

        // Embed Dockerfile and entrypoint at compile time
        let dockerfile_bytes = include_str!("sandbox.Dockerfile").as_bytes();
        let entrypoint_bytes = include_str!("sandbox-entrypoint.sh").as_bytes();

        // Create a tar archive with the Dockerfile and entrypoint
        let mut tar = tar::Builder::new(Vec::new());
        let mut append_bytes =
            |name: &str, bytes: &[u8]| -> Result<()> {
                let mut header = tar::Header::new_gnu();
                header.set_size(bytes.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append_data(&mut header, name, bytes)?;
                Ok(())
            };

        append_bytes("Dockerfile", &dockerfile_bytes)?;
        append_bytes("sandbox-entrypoint.sh", &entrypoint_bytes)?;
        let tar_bytes = tar.into_inner()?;

        // Build image
        let build_options: BuildImageOptions = BuildImageOptionsBuilder::new()
            .dockerfile("Dockerfile")
            .t(image_tag)
            .rm(true)
            .build();

        let body = Either::Left(Full::new(Bytes::from(tar_bytes)));
        let mut stream = self.docker.build_image(build_options, None, Some(body));

        // Wait for build to complete and log output
        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(stream) = output.stream {
                        tracing::info!("Docker build: {}", stream.trim());
                    }
                    if let Some(error) = output.error {
                        anyhow::bail!("Docker build error: {}", error);
                    }
                }
                Err(e) => anyhow::bail!("Docker build failed: {}", e),
            }
        }

        tracing::info!("Successfully built universal Docker image!");
        Ok(())
    }

    async fn prepare_workspace(&self, match_data: &Match, game: &Game) -> Result<String> {
        let workspace_dir = format!("/tmp/match_{}", match_data.id.replace(':', "_"));
        fs::create_dir_all(&workspace_dir).await?;

        // Write game server code to workspace from database
        if let Some(game_code) = &game.game_code {
            // Determine file extension based on game language
            let extension = match game.game_language.as_deref() {
                Some("rust") => "rs",
                Some("go") => "go",
                Some("c") => "c",
                Some("python") => "py",
                _ => "rs", // default to Rust
            };

            let dest_server = format!("{}/server.{}", workspace_dir, extension);
            fs::write(&dest_server, game_code)
                .await
                .context("Failed to write game server code to workspace")?;
            tracing::info!("Wrote game server code to {}", dest_server);
        } else {
            tracing::warn!(
                "Game {} has no game_code, container must provide server code",
                game.id
            );
        }

        // Fetch submission code from database and write to workspace
        for (idx, participant) in match_data.participants.iter().enumerate() {
            let submission_id = participant.submission_id.to_string();
            let submission = self
                .db_client
                .fetch_submission(&submission_id)
                .await
                .context(format!("Failed to fetch submission {}", submission_id))?;

            if let Some(code) = submission.code {
                let language = submission.language.as_deref().unwrap_or("rust");
                let ext = match language {
                    "rust" => "rs",
                    "go" => "go",
                    "c" => "c",
                    "cpp" => "cpp",
                    "python" => "py",
                    _ => "rs", // Default to Rust
                };
                let dest_file = format!("{}/player_{}.{}", workspace_dir, idx, ext);
                fs::write(&dest_file, code)
                    .await
                    .context(format!("Failed to write submission code to {}", dest_file))?;
                tracing::info!(
                    "Wrote submission {} code to {}",
                    participant.submission_id,
                    dest_file
                );
            } else {
                anyhow::bail!("Submission {} has no code", participant.submission_id);
            }
        }

        Ok(workspace_dir)
    }

    async fn run_match_container(
        &self,
        image_tag: &str,
        work_dir: &str,
        match_data: &Match,
        rounds: u32,
    ) -> Result<Vec<ParticipantResult>> {
        // Create container with resource limits
        let config = ContainerCreateBody {
            image: Some(image_tag.to_string()),
            env: Some(vec![format!("MATCH_ROUNDS={}", rounds)]),
            host_config: Some(HostConfig {
                binds: Some(vec![format!("{}:/workspace", work_dir)]), // Writable for compilation
                memory: Some(512 * 1024 * 1024),                       // 512MB
                nano_cpus: Some(1_000_000_000),                        // 1 CPU
                network_mode: Some("none".to_string()),                // No network access
                ..Default::default()
            }),
            working_dir: Some("/workspace".to_string()),
            ..Default::default()
        };

        let container_name = format!("match-{}", match_data.id.replace(':', "_"));
        let create_options: CreateContainerOptions = CreateContainerOptionsBuilder::new()
            .name(&container_name)
            .build();

        let container = self
            .docker
            .create_container(Some(create_options), config)
            .await?;

        // Start container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions>)
            .await?;

        // Wait for completion (with timeout) and capture exit code
        let timeout = std::time::Duration::from_secs(300); // 5 minutes
        let wait_result = tokio::time::timeout(timeout, async {
            let mut stream = self
                .docker
                .wait_container(&container.id, None::<WaitContainerOptions>);
            let mut exit_code: Option<i64> = None;
            while let Some(result) = stream.next().await {
                if let Ok(wait_result) = result {
                    exit_code = Some(wait_result.status_code);
                }
            }
            exit_code
        })
        .await;

        let exit_code = match wait_result {
            Ok(code) => code,
            Err(_) => {
                // Timeout - stop and remove container
                let _ = self
                    .docker
                    .stop_container(&container.id, None::<StopContainerOptions>)
                    .await;
                let _ = self
                    .docker
                    .remove_container(
                        &container.id,
                        Some(RemoveContainerOptionsBuilder::new().force(true).build()),
                    )
                    .await;
                anyhow::bail!("Match execution timed out after 5 minutes");
            }
        };

        // Get container logs (stdout contains match results in JSON)
        let mut logs_stream = self.docker.logs(
            &container.id,
            Some(LogsOptionsBuilder::new().stdout(true).stderr(true).build()),
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
                Some(RemoveContainerOptionsBuilder::new().force(true).build()),
            )
            .await?;

        // Parse results from logs
        if let Some(code) = exit_code {
            match code {
                1 => anyhow::bail!("GAME_CODE_COMPILATION_FAILED: {}", logs.trim()),
                2 => anyhow::bail!("PLAYER_CODE_COMPILATION_FAILED: {}", logs.trim()),
                _ => {}
            }
        }

        let results = self.parse_match_results(&logs, match_data)?;

        Ok(results)
    }

    fn parse_match_results(
        &self,
        logs: &str,
        match_data: &Match,
    ) -> Result<Vec<ParticipantResult>> {
        tracing::debug!("Raw container output:\n{}", logs);

        // Try JSON format first (backward compatibility)
        if let Ok(results) = self.parse_json_results(logs, match_data) {
            return Ok(results);
        }

        // Fall back to space/newline-separated format (as per GAME_SETTER_GUIDE.md)
        self.parse_simple_results(logs, match_data)
    }

    fn parse_json_results(&self, logs: &str, match_data: &Match) -> Result<Vec<ParticipantResult>> {
        // Expected format: JSON with scores for each submission_id
        // Try to find JSON in logs (may be mixed with other output)
        let json_start = logs.find('{');
        let json_end = logs.rfind('}');

        let json_str = if let (Some(start), Some(end)) = (json_start, json_end) {
            &logs[start..=end]
        } else {
            logs
        };

        let scores: HashMap<String, f64> = serde_json::from_str(json_str)?;

        let mut results: Vec<ParticipantResult> = match_data
            .participants
            .iter()
            .map(|p| {
                let sid = p.submission_id.to_string();
                let score = scores.get(&sid).copied().unwrap_or(0.0);
                ParticipantResult {
                    submission_id: sid,
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

    fn parse_simple_results(
        &self,
        logs: &str,
        match_data: &Match,
    ) -> Result<Vec<ParticipantResult>> {
        // Expected format: space or newline-separated scores/error codes
        // Examples: "100 85", "100\n85", "TLE 92", "100 WA RE"

        // Get the last non-empty line (final output)
        let last_line = logs
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .context("No output found in container logs")?;

        tracing::info!("Parsing results from: {}", last_line);

        // Split by whitespace
        let tokens: Vec<&str> = last_line.split_whitespace().collect();

        if tokens.len() != match_data.participants.len() {
            anyhow::bail!(
                "Expected {} scores but got {}. Output: {}",
                match_data.participants.len(),
                tokens.len(),
                last_line
            );
        }

        let mut results: Vec<ParticipantResult> = Vec::new();

        for (idx, (participant, token)) in match_data
            .participants
            .iter()
            .zip(tokens.iter())
            .enumerate()
        {
            let (score, metadata) = match token.to_uppercase().as_str() {
                "TLE" => (
                    0.0,
                    Some(serde_json::json!({"error": "Time Limit Exceeded"})),
                ),
                "WA" => (0.0, Some(serde_json::json!({"error": "Wrong Answer"}))),
                "RE" => (0.0, Some(serde_json::json!({"error": "Runtime Error"}))),
                "CE" => (0.0, Some(serde_json::json!({"error": "Compilation Error"}))),
                _ => {
                    // Try to parse as number
                    let score = token.parse::<f64>().context(format!(
                        "Invalid score/error code for player {}: {}",
                        idx + 1,
                        token
                    ))?;
                    (score, None)
                }
            };

            results.push(ParticipantResult {
                submission_id: participant.submission_id.to_string(),
                score,
                rank: None,
                is_winner: false,
                metadata,
            });
        }

        // Treat any error token (TLE/WA/RE/CE) as a failed match so it doesn't get a false "completed" status
        if results.iter().any(|r| r.metadata.is_some()) {
            let mut snippet = logs.trim();
            if snippet.len() > 2000 {
                snippet = &snippet[..2000];
            }
            anyhow::bail!(
                "Match returned error outputs: {}. Logs: {}",
                last_line,
                snippet.replace('\n', "\\n")
            );
        }

        // Determine rankings (errors get lowest rank)
        results.sort_by(|a, b| {
            // Error results go to the end
            match (a.metadata.is_some(), b.metadata.is_some()) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b
                    .score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        for (idx, result) in results.iter_mut().enumerate() {
            result.rank = Some((idx + 1) as u32);
            result.is_winner = idx == 0 && result.metadata.is_none();
        }

        Ok(results)
    }

    async fn cleanup_workspace(&self, work_dir: &str) -> Result<()> {
        fs::remove_dir_all(work_dir).await?;
        Ok(())
    }
}
