use landlock::*;
use std::path::Path;
use crate::sandbox::{Result, SandboxError};

pub fn apply_execution_rules(binary: &Path) -> Result<()> {
    let abi = ABI::V3;
    let mut ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| SandboxError::LandlockError(format!("Failed to create ruleset: {}", e)))?
        .create()
        .map_err(|e| SandboxError::LandlockError(format!("Failed to create ruleset: {}", e)))?
        .add_rule(PathBeneath::new(
            PathFd::new(binary)
                .map_err(|e| SandboxError::LandlockError(format!("Failed to open binary path: {}", e)))?,
            AccessFs::Execute | AccessFs::ReadFile | AccessFs::ReadDir,
        ))
        .map_err(|e| SandboxError::LandlockError(format!("Failed to add binary rule: {}", e)))?;

    tracing::debug!("Added Landlock rule for binary: {:?}", binary);

    if Path::new("/usr").exists() {
        ruleset = ruleset
            .add_rule(PathBeneath::new(
                PathFd::new("/usr")
                    .map_err(|e| SandboxError::LandlockError(format!("Failed to open /usr: {}", e)))?,
                AccessFs::from_read(abi) | AccessFs::Execute,
            ))
            .map_err(|e| SandboxError::LandlockError(format!("Failed to add /usr rule: {}", e)))?;
        tracing::debug!("Added Landlock rule for /usr (libraries)");
    }

    for lib_path in ["/lib", "/lib64"].iter() {
        if Path::new(lib_path).exists() {
            match PathFd::new(lib_path) {
                Ok(path_fd) => {
                    ruleset = ruleset
                        .add_rule(PathBeneath::new(
                            path_fd,
                            AccessFs::from_read(abi) | AccessFs::Execute,
                        ))
                        .map_err(|e| SandboxError::LandlockError(format!("Failed to add {} rule: {}", lib_path, e)))?;
                    tracing::debug!("Added Landlock rule for {}", lib_path);
                }
                Err(e) => {
                    tracing::debug!("Skipping Landlock rule for {} (not accessible): {}", lib_path, e);
                }
            }
        }
    }

    let status = ruleset
        .restrict_self()
        .map_err(|e| SandboxError::LandlockError(format!("Failed to restrict self: {}", e)))?;

    tracing::debug!("Landlock execution rules applied: {:?}", status);
    Ok(())
}

/// Check if landlock is supported on this system
pub fn is_supported() -> bool {
    // Try to create a simple ruleset to check if landlock is supported
    Ruleset::default()
        .handle_access(AccessFs::from_all(ABI::V3))
        .and_then(|r| r.create())
        .is_ok()
}
