use seccompiler::*;
use std::collections::BTreeMap;
use crate::services::sandbox::{Result, SandboxError};

pub fn apply_execution_filter() -> Result<()> {
    let filter = execution_filter()?;
    apply_filter(filter)
}

fn execution_filter() -> Result<BpfProgram> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    let allowed_syscalls = vec![
        nix::libc::SYS_read,
        nix::libc::SYS_write,
        nix::libc::SYS_writev,
        nix::libc::SYS_readv,
        nix::libc::SYS_pread64,
        nix::libc::SYS_pwrite64,
        nix::libc::SYS_lseek,
        nix::libc::SYS_ioctl,
        nix::libc::SYS_fcntl,
        nix::libc::SYS_open,
        nix::libc::SYS_openat,
        nix::libc::SYS_close,
        nix::libc::SYS_stat,
        nix::libc::SYS_fstat,
        nix::libc::SYS_lstat,
        nix::libc::SYS_newfstatat,
        nix::libc::SYS_statx,
        nix::libc::SYS_access,
        nix::libc::SYS_faccessat,
        nix::libc::SYS_faccessat2,
        nix::libc::SYS_readlink,
        nix::libc::SYS_readlinkat,
        nix::libc::SYS_mmap,
        nix::libc::SYS_mprotect,
        nix::libc::SYS_munmap,
        nix::libc::SYS_brk,
        nix::libc::SYS_rt_sigaction,
        nix::libc::SYS_rt_sigprocmask,
        nix::libc::SYS_rt_sigreturn,
        nix::libc::SYS_execve,
        nix::libc::SYS_exit,
        nix::libc::SYS_exit_group,
        nix::libc::SYS_getpid,
        nix::libc::SYS_getuid,
        nix::libc::SYS_geteuid,
        nix::libc::SYS_getgid,
        nix::libc::SYS_getegid,
        nix::libc::SYS_gettid,
        nix::libc::SYS_futex,
        nix::libc::SYS_clock_gettime,
        nix::libc::SYS_clock_nanosleep,
        nix::libc::SYS_nanosleep,
        nix::libc::SYS_getrandom,
        nix::libc::SYS_arch_prctl,
        nix::libc::SYS_set_tid_address,
        nix::libc::SYS_set_robust_list,
        nix::libc::SYS_prlimit64,
        nix::libc::SYS_getrlimit,
        nix::libc::SYS_rseq,
        nix::libc::SYS_uname,
        nix::libc::SYS_getcwd,
        nix::libc::SYS_getdents64,
        nix::libc::SYS_prctl,
        nix::libc::SYS_sched_getaffinity,
        nix::libc::SYS_sched_yield,
        nix::libc::SYS_poll,
        nix::libc::SYS_ppoll,
        nix::libc::SYS_select,
        nix::libc::SYS_pselect6,
        nix::libc::SYS_epoll_create,
        nix::libc::SYS_epoll_create1,
        nix::libc::SYS_epoll_ctl,
        nix::libc::SYS_epoll_wait,
        nix::libc::SYS_epoll_pwait,
        nix::libc::SYS_pipe,
        nix::libc::SYS_pipe2,
        nix::libc::SYS_dup,
        nix::libc::SYS_dup2,
        nix::libc::SYS_dup3,
    ];

    for syscall in allowed_syscalls {
        rules.insert(syscall, vec![]);
    }

    SeccompFilter::new(
        rules,
        SeccompAction::Errno(nix::libc::EPERM as u32),
        SeccompAction::Allow,
        std::env::consts::ARCH.try_into()
            .map_err(|e| SandboxError::SeccompError(format!("Invalid arch: {:?}", e)))?,
    )
    .map_err(|e| SandboxError::SeccompError(format!("Failed to create seccomp filter: {:?}", e)))?
    .try_into()
    .map_err(|e| SandboxError::SeccompError(format!("Failed to build BPF program: {:?}", e)))
}

fn apply_filter(program: BpfProgram) -> Result<()> {
    apply_filter_all_threads(&program)
        .map_err(|e| SandboxError::SeccompError(format!("Failed to apply seccomp filter: {:?}", e)))?;
    Ok(())
}
