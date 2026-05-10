// Sandbox: linux-primitive layer for running untrusted code under
// cgroups + namespaces + landlock + seccomp. Public surface is just
// `BuildSandbox`, `SandboxedBot`, and `ResourceLimits`. Everything else
// is internal.

// `cgroup` and `executor` are public because integration tests
// (judge/tests/pentest.rs) drive them directly. Everything else is a
// linux-primitive used only by this crate.
pub mod cgroup;
pub mod executor;
pub(crate) mod landlock;
pub(crate) mod namespace;
pub(crate) mod rootfs;
pub(crate) mod seccomp;

pub mod build;

pub use build::BuildSandbox;
pub use executor::{spawn_sandboxed, SandboxedBot, SandboxedProcess};

use std::fmt::Display;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SandboxError>;

/// Sandbox errors. Stage-tagged `Setup` for everything that can fail
/// during sandbox construction; a dedicated `CompilationTimeout` for
/// the only timeout the caller cares about distinguishing.
#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("{stage}: {msg}")]
    Setup { stage: &'static str, msg: String },

    #[error("compilation timeout (60s)")]
    CompilationTimeout,
}

impl SandboxError {
    pub fn setup(stage: &'static str, msg: impl Display) -> Self {
        Self::Setup {
            stage,
            msg: msg.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub memory_bytes: i64,
    pub cpu_quota: i64,
    pub cpu_period: u64,
    pub max_pids: i64,
}

impl ResourceLimits {
    /// 512MB RAM, 1 CPU, 128 processes.
    pub fn compilation() -> Self {
        Self {
            memory_bytes: 512 * 1024 * 1024,
            cpu_quota: 100_000,
            cpu_period: 100_000,
            max_pids: 128,
        }
    }

    /// 64MB RAM, 1 CPU, 16 processes.
    pub fn execution() -> Self {
        Self {
            memory_bytes: 64 * 1024 * 1024,
            cpu_quota: 100_000,
            cpu_period: 100_000,
            max_pids: 16,
        }
    }
}
