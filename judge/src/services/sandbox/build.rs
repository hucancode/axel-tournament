// Build sandbox: compiles a submission under cgroup limits with stdout
// + stderr captured to a per-submission log. Compilation is a trusted
// operation (we ship the toolchain) so we use cgroup-only isolation —
// no namespace, no landlock, no seccomp.

use crate::services::sandbox::cgroup::CgroupHandle;
use crate::services::sandbox::executor::redirect_stdio_to_log;
use crate::services::sandbox::{Result, SandboxError};
use nix::libc;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execvp, fork, ForkResult, Pid};
use std::ffi::CString;
use std::fs;
use std::os::fd::{AsFd, AsRawFd};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs as async_fs;

const COMPILE_TIMEOUT: Duration = Duration::from_secs(60);

pub struct BuildSandbox {
    workspace_root: PathBuf,
}

impl BuildSandbox {
    pub fn new(workspace_root: PathBuf) -> Result<Self> {
        if !workspace_root.exists() {
            fs::create_dir_all(&workspace_root)
                .map_err(|e| SandboxError::setup("build workspace create", e))?;
        }
        Ok(Self { workspace_root })
    }

    /// Compile `code` and return the output binary path. `submission_id`
    /// scopes the workspace and cgroup name.
    pub async fn compile(
        &self,
        submission_id: &str,
        language: &str,
        code: &str,
    ) -> Result<String> {
        let workspace = self.workspace_root.join(format!("submission_{}", submission_id));
        async_fs::create_dir_all(&workspace)
            .await
            .map_err(|e| SandboxError::setup("build workspace per-submission", e))?;

        let (source_file, binary_name) = match language {
            "rust" => ("main.rs", "player"),
            "go" => ("main.go", "player"),
            "c" => ("main.c", "player"),
            other => {
                return Err(SandboxError::setup(
                    "build language",
                    format!("unsupported language: {other}"),
                ))
            }
        };

        let source_path = workspace.join(source_file);
        async_fs::write(&source_path, code)
            .await
            .map_err(|e| SandboxError::setup("build write source", e))?;

        let (compiler_bin, args) = compiler_command(language, &workspace)?;

        tracing::info!(
            submission_id,
            language,
            workspace = %workspace.display(),
            "Starting compilation"
        );

        let (pid, _cgroup) = fork_and_compile(submission_id, &workspace, &compiler_bin, &args)?;

        let result = tokio::task::spawn_blocking(move || wait_for_child(pid, COMPILE_TIMEOUT))
            .await
            .map_err(|e| SandboxError::setup("build wait join", e))??;

        tracing::info!(submission_id, exit_code = result, "Compilation process exited");

        if result != 0 {
            let log_content = async_fs::read_to_string(workspace.join("compile.log"))
                .await
                .unwrap_or_else(|_| "Failed to read compile log".to_string());
            return Err(SandboxError::setup(
                "compile",
                format!("exit {result}. Log:\n{log_content}"),
            ));
        }

        let binary_path = workspace.join(binary_name);
        if !binary_path.exists() {
            return Err(SandboxError::setup("compile", "no binary produced"));
        }

        tracing::info!(
            submission_id,
            binary = %binary_path.display(),
            "Compilation succeeded"
        );
        Ok(binary_path.to_string_lossy().to_string())
    }
}

/// Fork the compiler process. Returns `(child_pid, cgroup)`. The cgroup
/// must outlive the child — drop kills any stragglers.
fn fork_and_compile(
    submission_id: &str,
    workspace: &Path,
    compiler_bin: &str,
    args: &[String],
) -> Result<(Pid, CgroupHandle)> {
    let log_path = workspace.join("compile.log");
    let log_file = fs::File::create(&log_path)
        .map_err(|e| SandboxError::setup("build log create", e))?;

    let cgroup = CgroupHandle::new_compilation(submission_id)?;

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // log_file drops here in the parent.
            cgroup.add_task(child).inspect_err(|e| {
                tracing::error!(submission_id, pid = %child, error = %e, "cgroup add_task");
            })?;
            tracing::debug!(submission_id, pid = %child, "Added child to cgroup");
            Ok((child, cgroup))
        }
        Ok(ForkResult::Child) => {
            redirect_stdio_to_log(log_file.as_fd()).ok();
            // Keep log_file alive past the dup2; OwnedFds inside
            // redirect_stdio_to_log are leaked so closing log_file
            // explicitly is safe.
            drop(log_file);

            std::env::set_current_dir(workspace).ok();

            let bin_cstring = CString::new(compiler_bin).expect("compiler path");
            let mut args_with_program = vec![bin_cstring.clone()];
            args_with_program.extend(args.iter().map(|a| CString::new(a.as_str()).unwrap()));

            execvp(&bin_cstring, &args_with_program).ok();
            std::process::exit(1);
        }
        Err(e) => Err(SandboxError::setup("fork", e)),
    }
}

/// Block on `waitpid` for at most `timeout`. SIGKILL on overrun.
fn wait_for_child(pid: Pid, timeout: Duration) -> Result<i32> {
    use std::os::unix::io::BorrowedFd;
    let pidfd = unsafe {
        libc::syscall(libc::SYS_pidfd_open, pid.as_raw(), 0u32) as i32
    };

    if pidfd >= 0 {
        let ready = poll_pidfd(pidfd, timeout);
        unsafe {
            libc::close(pidfd);
        }
        if !ready {
            tracing::warn!(pid = %pid, "Compilation timeout, killing process");
            let _ = kill(pid, Signal::SIGKILL);
            // Reap to avoid zombie.
            let _ = waitpid(pid, None);
            return Err(SandboxError::CompilationTimeout);
        }
        let _: BorrowedFd<'_>; // unused — keeps import warning quiet
    } else {
        // Fallback: spin with WNOHANG. Older kernels only.
        let start = std::time::Instant::now();
        while start.elapsed() <= timeout {
            match waitpid(pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::StillAlive) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                other => return interpret_status(other),
            }
        }
        let _ = kill(pid, Signal::SIGKILL);
        let _ = waitpid(pid, None);
        return Err(SandboxError::CompilationTimeout);
    }

    interpret_status(waitpid(pid, None))
}

fn poll_pidfd(pidfd: i32, timeout: Duration) -> bool {
    let mut pfd = libc::pollfd {
        fd: pidfd,
        events: libc::POLLIN,
        revents: 0,
    };
    let ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    let n = unsafe { libc::poll(&mut pfd, 1, ms) };
    n > 0
}

fn interpret_status(
    status: std::result::Result<WaitStatus, nix::Error>,
) -> Result<i32> {
    match status {
        Ok(WaitStatus::Exited(_, code)) => Ok(code),
        Ok(WaitStatus::Signaled(_, signal, _)) => Err(SandboxError::setup(
            "compile",
            format!("killed by signal: {:?}", signal),
        )),
        Ok(other) => Err(SandboxError::setup(
            "compile",
            format!("unexpected wait status: {:?}", other),
        )),
        Err(e) => Err(SandboxError::setup("waitpid", e)),
    }
}

fn compiler_path(env_var: &str, default: &str) -> String {
    std::env::var(env_var).unwrap_or_else(|_| default.to_string())
}

fn compiler_command(language: &str, workspace: &Path) -> Result<(String, Vec<String>)> {
    let output = workspace.join("player").to_string_lossy().to_string();
    match language {
        "rust" => {
            let source = workspace.join("main.rs").to_string_lossy().to_string();
            Ok((
                compiler_path("RUSTC_BIN", "rustc"),
                vec![
                    "--edition".into(),
                    "2024".into(),
                    "-C".into(),
                    "opt-level=2".into(),
                    "-o".into(),
                    output,
                    source,
                ],
            ))
        }
        "go" => {
            let source = workspace.join("main.go").to_string_lossy().to_string();
            Ok((
                compiler_path("GO_BIN", "go"),
                vec!["build".into(), "-o".into(), output, source],
            ))
        }
        "c" => {
            let source = workspace.join("main.c").to_string_lossy().to_string();
            Ok((
                compiler_path("GCC_BIN", "gcc"),
                vec!["-O2".into(), "-o".into(), output, source],
            ))
        }
        other => Err(SandboxError::setup(
            "build language",
            format!("unsupported: {other}"),
        )),
    }
}

// AsRawFd kept for back-compat with older callers; warn-clean.
#[allow(dead_code)]
fn _silence_unused<T: AsRawFd>(_: &T) {}
