// Run a compiled bot binary inside the sandbox + wire its stdio to a
// pair of pipes the orchestrator can drive. Two surfaces:
//
//   * `SandboxedBot` — async-friendly facade used by the room runtime.
//                      Drop = SIGTERM/SIGKILL the child + tear down cgroup.
//   * `spawn_sandboxed` — lower-level entry point returning raw fds and
//                         the cgroup handle. Used by integration tests
//                         that need direct waitpid/read control.

use crate::services::sandbox::cgroup::CgroupHandle;
use crate::services::sandbox::landlock;
use crate::services::sandbox::namespace::{
    create_namespaces, setup_mount_namespace, setup_self_uid_mapping,
};
use crate::services::sandbox::rootfs::setup_execution_rootfs;
use crate::services::sandbox::seccomp::apply_execution_filter;
use crate::services::sandbox::{Result, SandboxError};
use caps::{clear, CapSet};
use nix::sys::signal::{kill, Signal};
use nix::unistd::{dup2, execv, fork, pipe, ForkResult, Pid};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd};
use std::path::Path;
use std::time::Duration;
use tokio::fs::File as TokioFile;
use tokio::io::BufReader;

/// Raw sandbox spawn result: caller owns `stdin_fd`/`stdout_fd` (must
/// close them) and the cgroup. `pid` is also unmanaged — caller waits
/// or kills.
pub struct SandboxedProcess {
    pub pid: Pid,
    pub stdin_fd: i32,
    pub stdout_fd: i32,
    pub cgroup: CgroupHandle,
}

/// One sandboxed bot wired to its stdio pipes. Drop kills the
/// subprocess and tears down the cgroup.
pub struct SandboxedBot {
    pid: Option<Pid>,
    pub stdin: TokioFile,
    pub stdout: BufReader<TokioFile>,
    _cgroup: CgroupHandle,
}

impl SandboxedBot {
    /// Spawn `binary_path` under the execution sandbox.
    pub async fn spawn(player_id: &str, binary_path: &Path) -> Result<Self> {
        let label = player_id.to_string();
        let path = binary_path.to_path_buf();
        tokio::task::spawn_blocking(move || -> Result<SandboxedBot> {
            let raw = spawn_sandboxed(&label, &path)?;
            let stdin = unsafe { File::from_raw_fd(raw.stdin_fd) };
            let stdout = unsafe { File::from_raw_fd(raw.stdout_fd) };
            Ok(SandboxedBot {
                pid: Some(raw.pid),
                stdin: TokioFile::from_std(stdin),
                stdout: BufReader::new(TokioFile::from_std(stdout)),
                _cgroup: raw.cgroup,
            })
        })
        .await
        .map_err(|e| SandboxError::setup("bot spawn join", e))?
    }
}

impl Drop for SandboxedBot {
    fn drop(&mut self) {
        if let Some(pid) = self.pid.take() {
            let _ = kill(pid, Signal::SIGTERM);
            std::thread::sleep(Duration::from_millis(200));
            let _ = kill(pid, Signal::SIGKILL);
        }
    }
}

/// Spawn `binary_path` under cgroups + namespaces + landlock + seccomp
/// with stdio routed through pipes. Caller owns the returned fds.
pub fn spawn_sandboxed(player_id: &str, binary_path: &Path) -> Result<SandboxedProcess> {
    let (stdin_read, stdin_write) =
        pipe().map_err(|e| SandboxError::setup("pipe stdin", e))?;
    let (stdout_read, stdout_write) =
        pipe().map_err(|e| SandboxError::setup("pipe stdout", e))?;

    let stdin_read_raw = stdin_read.as_raw_fd();
    let stdout_write_raw = stdout_write.as_raw_fd();

    let cgroup = CgroupHandle::new_execution(player_id)?;

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            drop(stdin_read);
            drop(stdout_write);
            cgroup.add_task(child)?;

            tracing::info!(player_id, pid = %child, "Spawned sandboxed player process");
            Ok(SandboxedProcess {
                pid: child,
                stdin_fd: stdin_write.into_raw_fd(),
                stdout_fd: stdout_read.into_raw_fd(),
                cgroup,
            })
        }
        Ok(ForkResult::Child) => {
            drop(stdin_write);
            drop(stdout_read);

            if let Err(e) = child_setup_and_exec(binary_path, stdin_read_raw, stdout_write_raw) {
                eprintln!("Child setup failed: {}", e);
                std::process::exit(1);
            }
            unreachable!("execv should not return");
        }
        Err(e) => Err(SandboxError::setup("fork", e)),
    }
}

fn child_setup_and_exec(binary_path: &Path, stdin_fd: i32, stdout_fd: i32) -> Result<()> {
    redirect_stdio(stdin_fd, stdout_fd)?;

    let (host_uid, host_gid) = create_namespaces(false)?;
    setup_self_uid_mapping(host_uid, host_gid)?;
    setup_mount_namespace()?;

    let tmp_root = tempfile::tempdir()
        .map_err(|e| SandboxError::setup("rootfs tempdir", e))?;

    let binary_in_sandbox = setup_execution_rootfs(binary_path, tmp_root.path())?;

    if landlock::is_supported() {
        landlock::apply_execution_rules(&binary_in_sandbox)?;
    }

    drop_all_capabilities();
    apply_execution_filter()?;

    let bin_cstring = CString::new(binary_in_sandbox.to_str().unwrap())
        .map_err(|e| SandboxError::setup("invalid binary path", e))?;

    let args: Vec<CString> = vec![bin_cstring.clone()];
    execv(&bin_cstring, &args).map_err(|e| SandboxError::setup("execv", e))?;
    unreachable!("execv should not return");
}

/// Redirect stdin/stdout to the given fds; stderr stays attached to
/// the parent for debug output.
pub(crate) fn redirect_stdio(stdin_fd: i32, stdout_fd: i32) -> Result<()> {
    unsafe {
        let stdin_owned = OwnedFd::from_raw_fd(stdin_fd);
        let stdout_owned = OwnedFd::from_raw_fd(stdout_fd);
        let mut stdin = OwnedFd::from_raw_fd(0);
        let mut stdout = OwnedFd::from_raw_fd(1);
        dup2(&stdin_owned, &mut stdin).map_err(|e| SandboxError::setup("dup2 stdin", e))?;
        dup2(&stdout_owned, &mut stdout)
            .map_err(|e| SandboxError::setup("dup2 stdout", e))?;
        std::mem::forget(stdin);
        std::mem::forget(stdout);
    }
    Ok(())
}

/// Redirect stdout AND stderr to the same fd (capture-to-log).
pub(crate) fn redirect_stdio_to_log(log_fd: BorrowedFd<'_>) -> Result<()> {
    unsafe {
        let mut stdout = OwnedFd::from_raw_fd(1);
        let mut stderr = OwnedFd::from_raw_fd(2);
        dup2(&log_fd, &mut stdout).map_err(|e| SandboxError::setup("dup2 stdout->log", e))?;
        dup2(&log_fd, &mut stderr).map_err(|e| SandboxError::setup("dup2 stderr->log", e))?;
        std::mem::forget(stdout);
        std::mem::forget(stderr);
    }
    Ok(())
}

fn drop_all_capabilities() {
    let _ = clear(None, CapSet::Effective);
    let _ = clear(None, CapSet::Permitted);
    let _ = clear(None, CapSet::Inheritable);
    let _ = clear(None, CapSet::Ambient);
}
