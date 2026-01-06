use std::path::Path;
use std::os::unix::io::{FromRawFd, IntoRawFd, AsRawFd};
use caps::{CapSet, clear};
use nix::unistd::{fork, ForkResult, execv, Pid, pipe, dup2, close};
use std::ffi::CString;
use std::fs::File;
use crate::sandbox::{Result, SandboxError};
use crate::sandbox::namespace::{create_namespaces, setup_mount_namespace, setup_self_uid_mapping};
use crate::sandbox::cgroup::CgroupHandle;
use crate::sandbox::landlock::apply_execution_rules;
use crate::sandbox::seccomp::apply_execution_filter;
use crate::sandbox::rootfs::setup_execution_rootfs;

/// Result of spawning a sandboxed process
pub struct SandboxedProcess {
    pub pid: Pid,
    pub stdin_fd: i32,
    pub stdout_fd: i32,
    pub cgroup: CgroupHandle,
}

pub fn drop_all_capabilities() -> Result<()> {
    let _ = clear(None, CapSet::Effective);
    let _ = clear(None, CapSet::Permitted);
    let _ = clear(None, CapSet::Inheritable);
    let _ = clear(None, CapSet::Ambient);
    Ok(())
}
/// Spawn a binary in an isolated sandbox with stdin/stdout pipes
pub fn spawn_sandboxed(player_id: &str, binary_path: &Path) -> Result<SandboxedProcess> {
    // Create pipes for stdin/stdout communication
    let (stdin_read, stdin_write) = pipe()
        .map_err(|e| SandboxError::ProcessError(format!("Failed to create stdin pipe: {}", e)))?;

    let (stdout_read, stdout_write) = pipe()
        .map_err(|e| SandboxError::ProcessError(format!("Failed to create stdout pipe: {}", e)))?;

    let stdin_read_raw = stdin_read.as_raw_fd();
    let stdout_write_raw = stdout_write.as_raw_fd();

    // Keep pipes in blocking mode
    // The game uses async I/O with timeouts, so blocking is fine and avoids race conditions
    // where we try to read before the child process has started writing

    // Create cgroup before forking
    let cgroup = CgroupHandle::new_execution(player_id)?;

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // Parent process: close child ends of pipes
            drop(stdin_read);
            drop(stdout_write);

            // Add child to cgroup (do this before child enters new user namespace)
            cgroup.add_task(child)?;

            tracing::info!(
                player_id = %player_id,
                pid = %child,
                "Spawned sandboxed player process"
            );

            Ok(SandboxedProcess {
                pid: child,
                stdin_fd: stdin_write.into_raw_fd(),
                stdout_fd: stdout_read.into_raw_fd(),
                cgroup,
            })
        }
        Ok(ForkResult::Child) => {
            // Child process: setup sandbox and exec binary
            // Close parent ends of pipes (drop them)
            drop(stdin_write);
            drop(stdout_read);

            if let Err(e) = child_setup_and_exec(binary_path, stdin_read_raw, stdout_write_raw) {
                eprintln!("Child setup failed: {}", e);
                std::process::exit(1);
            }
            unreachable!("execv should not return");
        }
        Err(e) => Err(SandboxError::ProcessError(format!("Fork failed: {}", e))),
    }
}

/// Child process: setup sandbox and execute binary
fn child_setup_and_exec(
    binary_path: &Path,
    stdin_fd: i32,
    stdout_fd: i32,
) -> Result<()> {
    // 1. Dup pipes to stdin/stdout
    // Keep stderr (fd 2) pointing to parent's stderr for error messages
    dup2(stdin_fd, 0)
        .map_err(|e| SandboxError::ProcessError(format!("Failed to dup2 stdin: {}", e)))?;
    dup2(stdout_fd, 1)
        .map_err(|e| SandboxError::ProcessError(format!("Failed to dup2 stdout: {}", e)))?;

    // Close original pipe fds
    close(stdin_fd).ok();
    close(stdout_fd).ok();

    let (host_uid, host_gid) = create_namespaces(false)?;
    setup_self_uid_mapping(host_uid, host_gid)?;
    setup_mount_namespace()?;

    let tmp_root = tempfile::tempdir()
        .map_err(|e| SandboxError::RootfsError(format!("Failed to create temp dir: {}", e)))?;

    let binary_in_sandbox = setup_execution_rootfs(binary_path, tmp_root.path())?;

    if crate::sandbox::landlock::is_supported() {
        apply_execution_rules(&binary_in_sandbox)?;
    }

    drop_all_capabilities()?;
    apply_execution_filter()?;

    let bin_cstring = CString::new(binary_in_sandbox.to_str().unwrap())
        .map_err(|e| SandboxError::ProcessError(format!("Invalid binary path: {}", e)))?;

    let args: Vec<CString> = vec![bin_cstring.clone()];
    execv(&bin_cstring, &args)
        .map_err(|e| SandboxError::ProcessError(format!("execv failed: {}", e)))?;

    unreachable!("execv should not return");
}

/// Convert raw file descriptor to tokio-compatible File
pub fn fd_to_file(fd: i32) -> File {
    unsafe { File::from_raw_fd(fd) }
}
