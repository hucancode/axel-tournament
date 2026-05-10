use crate::services::sandbox::{Result, SandboxError};
use landlock::*;
use std::path::Path;

pub fn apply_execution_rules(binary: &Path) -> Result<()> {
    let abi = ABI::V3;
    let mut ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| SandboxError::setup("landlock ruleset", e))?
        .create()
        .map_err(|e| SandboxError::setup("landlock create", e))?
        .add_rule(PathBeneath::new(
            PathFd::new(binary).map_err(|e| SandboxError::setup("landlock open binary", e))?,
            AccessFs::Execute | AccessFs::ReadFile | AccessFs::ReadDir,
        ))
        .map_err(|e| SandboxError::setup("landlock add binary rule", e))?;

    tracing::debug!("Added Landlock rule for binary: {:?}", binary);

    if Path::new("/usr").exists() {
        ruleset = ruleset
            .add_rule(PathBeneath::new(
                PathFd::new("/usr")
                    .map_err(|e| SandboxError::setup("landlock open /usr", e))?,
                AccessFs::from_read(abi) | AccessFs::Execute,
            ))
            .map_err(|e| SandboxError::setup("landlock add /usr rule", e))?;
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
                        .map_err(|e| {
                            SandboxError::setup("landlock add lib rule", e)
                        })?;
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
        .map_err(|e| SandboxError::setup("landlock restrict_self", e))?;

    tracing::debug!("Landlock execution rules applied: {:?}", status);
    Ok(())
}

pub fn is_supported() -> bool {
    Ruleset::default()
        .handle_access(AccessFs::from_all(ABI::V3))
        .and_then(|r| r.create())
        .is_ok()
}
