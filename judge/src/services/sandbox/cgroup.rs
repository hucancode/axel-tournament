use crate::services::sandbox::{Result, ResourceLimits, SandboxError};
use cgroups_rs::fs::{cgroup_builder::CgroupBuilder, hierarchies, Cgroup, MaxValue};
use cgroups_rs::*;
use nix::unistd::Pid;

pub struct CgroupHandle {
    cgroup: Cgroup,
}

impl CgroupHandle {
    pub fn new_compilation(submission_id: &str) -> Result<Self> {
        Self::new(
            &format!("judge/compilation/submission_{}", submission_id),
            ResourceLimits::compilation(),
        )
    }

    pub fn new_execution(player_id: &str) -> Result<Self> {
        Self::new(
            &format!("judge/execution/player_{}", player_id),
            ResourceLimits::execution(),
        )
    }

    fn new(name: &str, limits: ResourceLimits) -> Result<Self> {
        let hierarchy = hierarchies::auto();
        let _ = Cgroup::load(hierarchy, name).delete();

        let cgroup: Cgroup = CgroupBuilder::new(name)
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
            .build(hierarchies::auto())
            .map_err(|e| SandboxError::setup("cgroup", e))?;

        Ok(Self { cgroup })
    }

    pub fn add_task(&self, pid: Pid) -> Result<()> {
        let raw_pid = pid.as_raw() as u64;
        self.cgroup
            .add_task_by_tgid(CgroupPid::from(raw_pid))
            .map_err(|e| SandboxError::setup("cgroup add_task", e))?;
        Ok(())
    }
}

impl Drop for CgroupHandle {
    fn drop(&mut self) {
        let _ = self.cgroup.kill();
        let _ = self.cgroup.delete();
    }
}
