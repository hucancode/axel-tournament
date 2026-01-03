use anyhow::{Result, anyhow};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, WaitContainerOptions};
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use futures_util::StreamExt;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

pub struct Compiler {
    docker: Docker,
    workspace_root: String,
    rust_image: String,
    go_image: String,
    c_image: String,
}

impl Compiler {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        let workspace_root = std::env::var("COMPILER_WORKSPACE")
            .unwrap_or_else(|_| "/tmp/compiler".to_string());

        let rust_image = std::env::var("RUST_COMPILER_IMAGE")
            .unwrap_or_else(|_| "rust:slim-trixie".to_string());
        let go_image = std::env::var("GO_COMPILER_IMAGE")
            .unwrap_or_else(|_| "golang:trixie".to_string());
        let c_image = std::env::var("C_COMPILER_IMAGE")
            .unwrap_or_else(|_| "gcc:trixie".to_string());

        Ok(Self {
            docker,
            workspace_root,
            rust_image,
            go_image,
            c_image,
        })
    }

    fn get_compiler_image(&self, language: &str) -> &str {
        match language {
            "rust" => &self.rust_image,
            "go" => &self.go_image,
            "c" => &self.c_image,
            _ => panic!("Unsupported language: {}", language),
        }
    }

    pub async fn compile_submission(
        &self,
        submission_id: &str,
        language: &str,
        code: &str,
    ) -> Result<String> {
        // Create workspace directory for this submission
        let workspace = PathBuf::from(&self.workspace_root)
            .join(format!("submission_{}", submission_id));
        fs::create_dir_all(&workspace).await?;

        // Write source code to file
        let source_file = match language {
            "rust" => "main.rs",
            "go" => "main.go",
            "c" => "main.c",
            _ => return Err(anyhow!("Unsupported language: {}", language)),
        };
        let source_path = workspace.join(source_file);
        fs::write(&source_path, code).await?;

        // Get compiler image
        let image = self.get_compiler_image(language);

        // Compile command based on language
        let (cmd, binary_name) = match language {
            "rust" => (
                vec!["rustc", "--edition", "2024", "-C", "opt-level=2", "-o", "/workspace/player", "/workspace/main.rs"],
                "player",
            ),
            "go" => (
                vec!["go", "build", "-o", "/workspace/player", "/workspace/main.go"],
                "player",
            ),
            "c" => (
                vec!["gcc", "-O2", "-o", "/workspace/player", "/workspace/main.c"],
                "player",
            ),
            _ => return Err(anyhow!("Unsupported language: {}", language)),
        };

        // Create container config
        let workspace_str = workspace.to_string_lossy().to_string();
        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(cmd.into_iter().map(String::from).collect()),
            working_dir: Some("/workspace".to_string()),
            host_config: Some(HostConfig {
                mounts: Some(vec![Mount {
                    target: Some("/workspace".to_string()),
                    source: Some(workspace_str.clone()),
                    typ: Some(MountTypeEnum::BIND),
                    ..Default::default()
                }]),
                network_mode: Some("none".to_string()),
                memory: Some(512 * 1024 * 1024), // 512MB
                nano_cpus: Some(1_000_000_000), // 1 CPU
                ..Default::default()
            }),
            ..Default::default()
        };

        // Create and start container
        let sanitized_id = submission_id.replace(":", "_");
        let container_name = format!("compiler_{}_{}", language, sanitized_id);
        let create_options = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let container = self.docker.create_container(Some(create_options), config).await?;
        info!("Created compilation container: {}", container.id);

        self.docker.start_container::<String>(&container.id, None).await?;

        // Wait for container with timeout
        let timeout = tokio::time::Duration::from_secs(60);
        let wait_result = tokio::time::timeout(
            timeout,
            async {
                let mut wait_stream = self.docker.wait_container::<String>(
                    &container.id,
                    Some(WaitContainerOptions {
                        condition: "not-running".to_string(),
                    }),
                );
                wait_stream.next().await
            },
        )
        .await;

        // Remove container
        let _ = self.docker.remove_container(
            &container.id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        ).await;

        // Check compilation result
        match wait_result {
            Ok(Some(Ok(result))) => {
                if result.status_code != 0 {
                    return Err(anyhow!("Compilation failed with exit code: {}", result.status_code));
                }
            }
            Ok(Some(Err(e))) => {
                return Err(anyhow!("Container wait error: {}", e));
            }
            Ok(None) => {
                return Err(anyhow!("Container wait stream ended unexpectedly"));
            }
            Err(_) => {
                return Err(anyhow!("Compilation timeout (60s)"));
            }
        }

        // Check if binary was created
        let binary_path = workspace.join(binary_name);
        if !binary_path.exists() {
            return Err(anyhow!("Compilation produced no binary"));
        }

        // Return path to compiled binary
        Ok(binary_path.to_string_lossy().to_string())
    }
}
