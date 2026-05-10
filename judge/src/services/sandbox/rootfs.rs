use crate::services::sandbox::{Result, SandboxError};
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::unistd::{chdir, pivot_root};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

fn check_if_dynamic_binary(binary: &Path) -> Result<bool> {
    let mut file = fs::File::open(binary)
        .map_err(|e| SandboxError::setup("rootfs open binary", e))?;

    let mut header = [0u8; 64];
    file.read_exact(&mut header)
        .map_err(|e| SandboxError::setup("rootfs read ELF header", e))?;

    if &header[0..4] != b"\x7fELF" {
        return Ok(false);
    }

    let is_64bit = header[4] == 2;
    let e_phoff = if is_64bit {
        u64::from_le_bytes(header[32..40].try_into().unwrap())
    } else {
        u32::from_le_bytes(header[28..32].try_into().unwrap()) as u64
    };

    if e_phoff == 0 {
        return Ok(false);
    }

    let mut phdr = vec![0u8; if is_64bit { 56 } else { 32 }];
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(e_phoff))
        .map_err(|e| SandboxError::setup("rootfs seek program headers", e))?;

    let e_phnum = u16::from_le_bytes(header[56..58].try_into().unwrap_or([0, 0]));
    for _ in 0..e_phnum.min(64) {
        file.read_exact(&mut phdr)
            .map_err(|e| SandboxError::setup("rootfs read program header", e))?;

        let p_type = u32::from_le_bytes(phdr[0..4].try_into().unwrap());
        if p_type == 3 {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn setup_execution_rootfs(binary: &Path, tmp_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(tmp_dir.join("dev"))
        .map_err(|e| SandboxError::setup("rootfs create dev", e))?;
    fs::create_dir_all(tmp_dir.join("oldroot"))
        .map_err(|e| SandboxError::setup("rootfs create oldroot", e))?;

    let binary_dest = tmp_dir.join("player");
    fs::copy(binary, &binary_dest)
        .map_err(|e| SandboxError::setup("rootfs copy binary", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_dest)
            .map_err(|e| SandboxError::setup("rootfs binary metadata", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_dest, perms)
            .map_err(|e| SandboxError::setup("rootfs binary perms", e))?;
    }

    if check_if_dynamic_binary(&binary_dest)? {
        tracing::debug!("Binary is dynamically linked, mounting system libraries");
        fs::create_dir_all(tmp_dir.join("usr"))
            .map_err(|e| SandboxError::setup("rootfs create usr", e))?;

        mount(
            Some(Path::new("/usr")),
            tmp_dir.join("usr").as_path(),
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        )
        .map_err(|e| SandboxError::setup("rootfs bind /usr", e))?;

        mount(
            None::<&str>,
            tmp_dir.join("usr").as_path(),
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY,
            None::<&str>,
        )
        .map_err(|e| SandboxError::setup("rootfs remount /usr ro", e))?;

        std::os::unix::fs::symlink("usr/lib", tmp_dir.join("lib"))
            .map_err(|e| SandboxError::setup("rootfs symlink /lib", e))?;

        if Path::new("/usr/lib64").exists() {
            std::os::unix::fs::symlink("usr/lib64", tmp_dir.join("lib64"))
                .map_err(|e| SandboxError::setup("rootfs symlink /lib64", e))?;
        } else {
            std::os::unix::fs::symlink("usr/lib", tmp_dir.join("lib64"))
                .map_err(|e| SandboxError::setup("rootfs symlink /lib64 fallback", e))?;
        }
    } else {
        tracing::debug!("Binary is statically linked");
    }

    mount(
        Some(tmp_dir),
        tmp_dir,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| SandboxError::setup("rootfs bind new root", e))?;

    pivot_root(tmp_dir, &tmp_dir.join("oldroot"))
        .map_err(|e| SandboxError::setup("rootfs pivot_root", e))?;

    chdir("/").map_err(|e| SandboxError::setup("rootfs chdir /", e))?;

    umount2("/oldroot", MntFlags::MNT_DETACH)
        .map_err(|e| SandboxError::setup("rootfs umount oldroot", e))?;

    fs::remove_dir("/oldroot")
        .map_err(|e| SandboxError::setup("rootfs remove oldroot", e))?;

    tracing::debug!("Execution rootfs setup complete");
    Ok(PathBuf::from("/player"))
}
