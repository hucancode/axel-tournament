use seccompiler::*;
use std::collections::BTreeMap;
use crate::services::sandbox::{Result, SandboxError};

pub fn apply_execution_filter() -> Result<()> {
    let filter = execution_filter()?;
    apply_filter(filter)
}

/// Whitelisted syscalls for sandboxed execution. Anything not on this
/// list is denied with EPERM. Public so an integration test can
/// snapshot the set and catch accidental drift during refactors.
pub fn allowed_syscalls() -> Vec<i64> {
    vec![
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
    ]
}

fn execution_filter() -> Result<BpfProgram> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
    for syscall in allowed_syscalls() {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Drift detector: deny-list expansions and accidental removals
    /// both change this count. Bump intentionally when the policy
    /// changes — the bump is a code-review hook.
    #[test]
    fn allowlist_size_is_locked() {
        let n = allowed_syscalls().len();
        assert_eq!(
            n, 69,
            "syscall allowlist size changed (now {n}). \
             Review the diff: each addition expands attack surface, \
             each removal may break legitimate runtimes. Update the \
             expected count after intentional policy change."
        );
    }

    #[test]
    fn allowlist_has_no_duplicates() {
        let mut v = allowed_syscalls();
        let before = v.len();
        v.sort();
        v.dedup();
        assert_eq!(v.len(), before, "duplicate syscall numbers in allowlist");
    }

    /// Block-list: syscalls we never want to allow even by accident.
    /// Adding any of these would let user code escape the sandbox.
    #[test]
    fn allowlist_excludes_dangerous_syscalls() {
        let allowed = allowed_syscalls();
        let banned = [
            ("ptrace", nix::libc::SYS_ptrace),
            ("mount", nix::libc::SYS_mount),
            ("setuid", nix::libc::SYS_setuid),
            ("setgid", nix::libc::SYS_setgid),
            ("seteuid", nix::libc::SYS_setresuid),
            ("clone", nix::libc::SYS_clone),
            ("fork", nix::libc::SYS_fork),
            ("vfork", nix::libc::SYS_vfork),
            ("socket", nix::libc::SYS_socket),
            ("connect", nix::libc::SYS_connect),
            ("bind", nix::libc::SYS_bind),
            ("listen", nix::libc::SYS_listen),
            ("accept", nix::libc::SYS_accept),
            ("sendto", nix::libc::SYS_sendto),
            ("recvfrom", nix::libc::SYS_recvfrom),
            ("unshare", nix::libc::SYS_unshare),
            ("setns", nix::libc::SYS_setns),
            ("kexec_load", nix::libc::SYS_kexec_load),
            ("init_module", nix::libc::SYS_init_module),
            ("delete_module", nix::libc::SYS_delete_module),
            ("reboot", nix::libc::SYS_reboot),
            ("bpf", nix::libc::SYS_bpf),
            ("perf_event_open", nix::libc::SYS_perf_event_open),
            ("kill", nix::libc::SYS_kill),
            ("tkill", nix::libc::SYS_tkill),
            ("tgkill", nix::libc::SYS_tgkill),
            ("process_vm_readv", nix::libc::SYS_process_vm_readv),
            ("process_vm_writev", nix::libc::SYS_process_vm_writev),
        ];
        for (name, num) in banned {
            assert!(
                !allowed.contains(&num),
                "dangerous syscall {name} (#{num}) is in allowlist"
            );
        }
    }
}
