use seccompiler::*;
use std::collections::BTreeMap;
use crate::sandbox::{Result, SandboxError};

pub fn apply_execution_filter() -> Result<()> {
    let filter = execution_filter()?;
    apply_filter(filter)
}

fn execution_filter() -> Result<BpfProgram> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    let allowed_syscalls = vec![
        libc::SYS_read,
        libc::SYS_write,
        libc::SYS_writev,
        libc::SYS_readv,
        libc::SYS_pread64,
        libc::SYS_pwrite64,
        libc::SYS_lseek,
        libc::SYS_ioctl,
        libc::SYS_fcntl,
        libc::SYS_open,
        libc::SYS_openat,
        libc::SYS_close,
        libc::SYS_stat,
        libc::SYS_fstat,
        libc::SYS_lstat,
        libc::SYS_newfstatat,
        libc::SYS_statx,
        libc::SYS_access,
        libc::SYS_faccessat,
        libc::SYS_faccessat2,
        libc::SYS_readlink,
        libc::SYS_readlinkat,
        libc::SYS_mmap,
        libc::SYS_mprotect,
        libc::SYS_munmap,
        libc::SYS_brk,
        libc::SYS_rt_sigaction,
        libc::SYS_rt_sigprocmask,
        libc::SYS_rt_sigreturn,
        libc::SYS_execve,
        libc::SYS_exit,
        libc::SYS_exit_group,
        libc::SYS_getpid,
        libc::SYS_getuid,
        libc::SYS_geteuid,
        libc::SYS_getgid,
        libc::SYS_getegid,
        libc::SYS_gettid,
        libc::SYS_futex,
        libc::SYS_clock_gettime,
        libc::SYS_clock_nanosleep,
        libc::SYS_nanosleep,
        libc::SYS_getrandom,
        libc::SYS_arch_prctl,
        libc::SYS_set_tid_address,
        libc::SYS_set_robust_list,
        libc::SYS_prlimit64,
        libc::SYS_getrlimit,
        libc::SYS_rseq,
        libc::SYS_uname,
        libc::SYS_getcwd,
        libc::SYS_getdents64,
        libc::SYS_prctl,
        libc::SYS_sched_getaffinity,
        libc::SYS_sched_yield,
        libc::SYS_poll,
        libc::SYS_ppoll,
        libc::SYS_select,
        libc::SYS_pselect6,
        libc::SYS_epoll_create,
        libc::SYS_epoll_create1,
        libc::SYS_epoll_ctl,
        libc::SYS_epoll_wait,
        libc::SYS_epoll_pwait,
        libc::SYS_pipe,
        libc::SYS_pipe2,
        libc::SYS_dup,
        libc::SYS_dup2,
        libc::SYS_dup3,
    ];

    for syscall in allowed_syscalls {
        rules.insert(syscall, vec![]);
    }

    SeccompFilter::new(
        rules,
        SeccompAction::Errno(libc::EPERM as u32),
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
