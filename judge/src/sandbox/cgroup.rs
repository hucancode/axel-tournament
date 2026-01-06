use cgroups_rs::*;
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::MaxValue;
use nix::unistd::Pid;
use crate::sandbox::{Result, SandboxError, ResourceLimits};

pub struct CgroupHandle {
    cgroup: Cgroup,
}

impl CgroupHandle {
    /// Create a new cgroup for compilation with appropriate resource limits
    pub fn new_compilation(submission_id: &str) -> Result<Self> {
        let limits = ResourceLimits::compilation();
        Self::new(&format!("judge/compilation/submission_{}", submission_id), limits)
    }

    /// Create a new cgroup for execution with appropriate resource limits
    pub fn new_execution(player_id: &str) -> Result<Self> {
        let limits = ResourceLimits::execution();
        Self::new(&format!("judge/execution/player_{}", player_id), limits)
    }

    /// Create a cgroup with custom resource limits
    fn new(cgroup_name: &str, limits: ResourceLimits) -> Result<Self> {
        let hierarchy = cgroups_rs::hierarchies::auto();

        let cgroup: Cgroup = CgroupBuilder::new(cgroup_name)
            .memory()
                .memory_hard_limit(limits.memory_bytes)
                .done()
            .cpu()
                .quota(limits.cpu_quota)
                .period(limits.cpu_period)
                .done()
            .pid()
                .maximum_number_of_processes(MaxValue::Value(limits.max_pids))
                .done()
            .build(hierarchy)
            .map_err(|e| SandboxError::CgroupError(format!("Failed to create cgroup: {}", e)))?;

        Ok(Self { cgroup })
    }

    pub fn add_task(&self, pid: Pid) -> Result<()> {
        let raw_pid = pid.as_raw() as u64;
        self.cgroup
            .add_task_by_tgid(CgroupPid::from(raw_pid))
            .map_err(|e| SandboxError::CgroupError(format!("Failed to add task to cgroup v2: {}", e)))?;
        Ok(())
    }
}

impl Drop for CgroupHandle {
    fn drop(&mut self) {
        let _ = self.cgroup.kill();
        let _ = self.cgroup.delete();
    }
}
