use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as async_fs;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::sys::signal::{kill, Signal};
use nix::unistd::{fork, ForkResult, execvp, Pid};
use std::os::unix::io::IntoRawFd;
use std::ffi::CString;
use std::time::Duration;
use crate::sandbox::{Result, SandboxError};
use crate::sandbox::cgroup::CgroupHandle;

/// Compiler sandbox for secure code compilation
pub struct CompilerSandbox {
    workspace_root: PathBuf,
}

impl CompilerSandbox {
    /// Create a new compiler sandbox
    pub fn new(workspace_root: PathBuf) -> Result<Self> {
        if !workspace_root.exists() {
            fs::create_dir_all(&workspace_root)
                .map_err(|e| SandboxError::CompilationError(format!("Failed to create workspace: {}", e)))?;
        }

        Ok(Self { workspace_root })
    }

    /// Compile source code with resource limits (cgroups only, no namespace isolation)
    /// Compilation doesn't need namespace isolation since the compiler is trusted.
    /// We only need CPU/memory limits to prevent resource exhaustion.
    pub async fn compile(
        &self,
        submission_id: &str,
        language: &str,
        code: &str,
    ) -> Result<String> {
        // Create workspace directory for this submission
        let workspace = self.workspace_root.join(format!("submission_{}", submission_id));
        async_fs::create_dir_all(&workspace).await
            .map_err(|e| SandboxError::CompilationError(format!("Failed to create submission workspace: {}", e)))?;

        // Write source code to file
        let (source_file, binary_name) = match language {
            "rust" => ("main.rs", "player"),
            "go" => ("main.go", "player"),
            "c" => ("main.c", "player"),
            _ => return Err(SandboxError::CompilationError(format!("Unsupported language: {}", language))),
        };

        let source_path = workspace.join(source_file);
        async_fs::write(&source_path, code).await
            .map_err(|e| SandboxError::CompilationError(format!("Failed to write source file: {}", e)))?;

        // Get compiler command
        let (compiler_bin, args) = self.get_compiler_command(language, &workspace)?;

        tracing::info!(
            submission_id = %submission_id,
            language = %language,
            workspace = %workspace.display(),
            "Starting compilation"
        );

        // Fork and execute with cgroup limits only
        let (pid, _cgroup) = match self.fork_and_compile_simple(
            submission_id,
            &workspace,
            &compiler_bin,
            &args,
        ) {
            Ok((pid, cgroup)) => {
                tracing::info!(
                    submission_id = %submission_id,
                    pid = %pid,
                    "Forked compiler process"
                );
                (pid, cgroup)
            }
            Err(e) => {
                tracing::error!(
                    submission_id = %submission_id,
                    error = %e,
                    "Failed to fork compiler process"
                );
                return Err(e);
            }
        };

        // Wait for compilation with timeout (in separate task to avoid blocking)
        let timeout = Duration::from_secs(60);
        let result = tokio::task::spawn_blocking(move || {
            Self::wait_for_child(pid, timeout)
        })
        .await
        .map_err(|e| SandboxError::CompilationError(format!("Task join error: {}", e)))??;

        // Cgroup will be automatically cleaned up here when _cgroup is dropped

        tracing::info!(
            submission_id = %submission_id,
            exit_code = result,
            "Compilation process exited"
        );

        if result != 0 {
            // Read compiler log for error details
            let log_path = workspace.join("compile.log");
            let log_content = async_fs::read_to_string(&log_path).await
                .unwrap_or_else(|_| "Failed to read compile log".to_string());

            return Err(SandboxError::CompilationError(format!(
                "Compilation failed with exit code: {}. Log:\n{}",
                result,
                log_content
            )));
        }

        // Check if binary was created
        let binary_path = workspace.join(binary_name);
        if !binary_path.exists() {
            return Err(SandboxError::CompilationError("Compilation produced no binary".to_string()));
        }

        tracing::info!(
            submission_id = %submission_id,
            binary = %binary_path.display(),
            "Compilation succeeded"
        );

        Ok(binary_path.to_string_lossy().to_string())
    }

    /// Fork and execute compiler with cgroup limits only (no namespace isolation)
    /// This is simpler and avoids the rustup/user-namespace issue.
    /// Returns (Pid, CgroupHandle) - the cgroup must be kept alive until compilation finishes.
    fn fork_and_compile_simple(
        &self,
        submission_id: &str,
        workspace: &Path,
        compiler_bin: &str,
        args: &[String],
    ) -> Result<(Pid, CgroupHandle)> {
        // Create output log file for compiler stderr/stdout
        let log_path = workspace.join("compile.log");
        let log_file = fs::File::create(&log_path)
            .map_err(|e| SandboxError::ProcessError(format!("Failed to create log file: {}", e)))?;
        let log_fd = log_file.into_raw_fd();

        // Create cgroup before forking
        let cgroup = CgroupHandle::new_compilation(submission_id)?;

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent: close log fd
                nix::unistd::close(log_fd).ok();

                // Parent: add child to cgroup for resource limits
                if let Err(e) = cgroup.add_task(child) {
                    tracing::error!(
                        submission_id = %submission_id,
                        pid = %child,
                        error = %e,
                        "Failed to add child to cgroup"
                    );
                    return Err(e);
                }

                tracing::debug!(
                    submission_id = %submission_id,
                    pid = %child,
                    "Added child to cgroup"
                );

                // Return both pid and cgroup handle
                // The cgroup must stay alive until compilation finishes
                Ok((child, cgroup))
            }
            Ok(ForkResult::Child) => {
                // Child process: redirect stdout and stderr to log file
                use std::os::fd::{FromRawFd, OwnedFd};
                unsafe {
                    let log_owned = OwnedFd::from_raw_fd(log_fd);
                    let mut stdout = OwnedFd::from_raw_fd(1);
                    let mut stderr = OwnedFd::from_raw_fd(2);
                    nix::unistd::dup2(&log_owned, &mut stdout).ok();
                    nix::unistd::dup2(&log_owned, &mut stderr).ok();
                    std::mem::forget(stdout);
                    std::mem::forget(stderr);
                    // log_owned will be closed when dropped
                }

                // Change to workspace directory
                std::env::set_current_dir(workspace).ok();

                // Execute compiler directly (no namespace isolation needed)
                // Use execvp which searches PATH
                let bin_cstring = CString::new(compiler_bin)
                    .map_err(|e| SandboxError::ProcessError(format!("Invalid compiler path: {}", e)))
                    .unwrap();

                // execvp expects argv[0] to be the program name
                let mut args_with_program = vec![CString::new(compiler_bin).unwrap()];
                args_with_program.extend(
                    args.iter().map(|arg| CString::new(arg.as_str()).unwrap())
                );

                execvp(&bin_cstring, &args_with_program).ok();
                std::process::exit(1);
            }
            Err(e) => Err(SandboxError::ProcessError(format!("Fork failed: {}", e))),
        }
    }

    /// Wait for child process with timeout
    fn wait_for_child(pid: Pid, timeout: Duration) -> Result<i32> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                // Timeout: kill the process
                tracing::warn!(pid = %pid, "Compilation timeout, killing process");
                let _ = kill(pid, Signal::SIGKILL);
                return Err(SandboxError::CompilationTimeout);
            }

            // Non-blocking wait
            match waitpid(pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(_, code)) => {
                    return Ok(code);
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    return Err(SandboxError::CompilationError(
                        format!("Process killed by signal: {:?}", signal)
                    ));
                }
                Ok(WaitStatus::StillAlive) => {
                    // Still running, sleep briefly
                    std::thread::sleep(Duration::from_millis(100));
                }
                Ok(status) => {
                    return Err(SandboxError::CompilationError(
                        format!("Unexpected wait status: {:?}", status)
                    ));
                }
                Err(e) => {
                    return Err(SandboxError::ProcessError(format!("waitpid failed: {}", e)));
                }
            }
        }
    }

    fn get_compiler_path(env_var: &str, default: &str) -> String {
        std::env::var(env_var).unwrap_or_else(|_| default.to_string())
    }

    /// Get compiler binary and arguments for the given language
    /// Now uses actual filesystem paths since we're not in a namespace
    fn get_compiler_command(&self, language: &str, workspace: &Path) -> Result<(String, Vec<String>)> {
        match language {
            "rust" => {
                let rustc = Self::get_compiler_path("RUSTC_BIN", "rustc");
                let output_path = workspace.join("player");
                let source_path = workspace.join("main.rs");

                Ok((
                    rustc,
                    vec![
                        "--edition".to_string(),
                        "2024".to_string(),
                        "-C".to_string(),
                        "opt-level=2".to_string(),
                        "-o".to_string(),
                        output_path.to_string_lossy().to_string(),
                        source_path.to_string_lossy().to_string(),
                    ],
                ))
            }
            "go" => {
                let go = Self::get_compiler_path("GO_BIN", "go");
                let output_path = workspace.join("player");
                let source_path = workspace.join("main.go");

                Ok((
                    go,
                    vec![
                        "build".to_string(),
                        "-o".to_string(),
                        output_path.to_string_lossy().to_string(),
                        source_path.to_string_lossy().to_string(),
                    ],
                ))
            }
            "c" => {
                let gcc = Self::get_compiler_path("GCC_BIN", "gcc");
                let output_path = workspace.join("player");
                let source_path = workspace.join("main.c");

                Ok((
                    gcc,
                    vec![
                        "-O2".to_string(),
                        "-o".to_string(),
                        output_path.to_string_lossy().to_string(),
                        source_path.to_string_lossy().to_string(),
                    ],
                ))
            }
            _ => Err(SandboxError::CompilationError(format!("Unsupported language: {}", language))),
        }
    }
}
