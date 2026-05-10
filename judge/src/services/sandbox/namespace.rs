use crate::services::sandbox::{Result, SandboxError};
use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{Gid, Uid};
use std::fs;

pub fn create_namespaces(include_network: bool) -> Result<(u32, u32)> {
    let host_uid = Uid::current().as_raw();
    let host_gid = Gid::current().as_raw();

    let mut flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWUSER
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWUTS;

    if include_network {
        flags |= CloneFlags::CLONE_NEWNET;
    }

    unshare(flags).map_err(|e| SandboxError::setup("unshare", e))?;
    Ok((host_uid, host_gid))
}

pub fn setup_mount_namespace() -> Result<()> {
    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None::<&str>,
    )
    .map_err(|e| SandboxError::setup("mount root private", e))?;
    Ok(())
}

pub fn setup_self_uid_mapping(host_uid: u32, host_gid: u32) -> Result<()> {
    let setgroups_path = "/proc/self/setgroups";
    if let Err(e) = fs::write(setgroups_path, "deny") {
        if let Ok(content) = fs::read_to_string(setgroups_path) {
            if content.trim() != "deny" {
                tracing::warn!("Failed to deny setgroups: {}", e);
            }
        }
    }

    fs::write("/proc/self/uid_map", format!("1000 {} 1", host_uid))
        .map_err(|e| SandboxError::setup("write uid_map", e))?;
    fs::write("/proc/self/gid_map", format!("1000 {} 1", host_gid))
        .map_err(|e| SandboxError::setup("write gid_map", e))?;
    Ok(())
}
