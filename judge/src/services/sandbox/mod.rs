pub mod namespace;
pub mod cgroup;
pub mod landlock;
pub mod seccomp;
pub mod rootfs;
pub mod compiler;
pub mod executor;

use thiserror::Error;

/// Sandbox result type
pub type Result<T> = std::result::Result<T, SandboxError>;

/// Sandbox error types
#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to create namespace: {0}")]
    NamespaceError(String),

    #[error("Failed to setup cgroup: {0}")]
    CgroupError(String),

    #[error("Failed to setup rootfs: {0}")]
    RootfsError(String),

    #[error("Failed to apply landlock rules: {0}")]
    LandlockError(String),

    #[error("Failed to apply seccomp filter: {0}")]
    SeccompError(String),

    #[error("Process execution failed: {0}")]
    ProcessError(String),

    #[error("Compilation failed: {0}")]
    CompilationError(String),

    #[error("Compilation timeout (60s)")]
    CompilationTimeout,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Nix error: {0}")]
    NixError(#[from] nix::Error),

    #[error("Cgroups error: {0}")]
    CgroupsError(#[from] cgroups_rs::fs::error::Error),

    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Configuration for sandbox resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Memory limit in bytes
    pub memory_bytes: i64,
    /// CPU quota (100000 = 1 CPU)
    pub cpu_quota: i64,
    /// CPU period (usually 100000 microseconds)
    pub cpu_period: u64,
    /// Maximum number of PIDs
    pub max_pids: i64,
}

impl ResourceLimits {
    /// Limits for compilation: 512MB RAM, 1 CPU, 128 processes
    pub fn compilation() -> Self {
        Self {
            memory_bytes: 512 * 1024 * 1024,
            cpu_quota: 100_000,
            cpu_period: 100_000,
            max_pids: 128,
        }
    }

    /// Limits for execution: 64MB RAM, 1 CPU, 16 processes
    pub fn execution() -> Self {
        Self {
            memory_bytes: 64 * 1024 * 1024,
            cpu_quota: 100_000,
            cpu_period: 100_000,
            max_pids: 16,
        }
    }
}
