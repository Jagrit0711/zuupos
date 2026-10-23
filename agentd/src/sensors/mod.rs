//! Sensors — real hardware telemetry for ZuuposOS.
//!
//! Reads CPU, memory, battery and load average via the `sysinfo` crate.
//! Returns a `SystemMetrics` snapshot that feeds into the agent's perception.

use crate::ipc::SystemMetrics;
use log::debug;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct SensorHub {
    sys: System,
}

impl SensorHub {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        Self { sys }
    }

    /// Refresh hardware readings and return a snapshot.
    pub fn read(&mut self) -> SystemMetrics {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        let cpu_usage = self.sys.global_cpu_usage();

        let mem_total_mb = self.sys.total_memory() / 1_048_576;
        let mem_used_mb = self.sys.used_memory() / 1_048_576;

        // Battery via system info (requires components, best-effort)
        let battery_pct = None; // Will be wired to battery crate in Phase 2

        let load_avg_1m = System::load_average().one as f32;

        debug!(
            "[sensors] CPU: {:.1}% | Mem: {}/{} MB | Load: {:.2}",
            cpu_usage, mem_used_mb, mem_total_mb, load_avg_1m
        );

        SystemMetrics {
            cpu_usage,
            mem_used_mb,
            mem_total_mb,
            battery_pct,
            load_avg_1m,
        }
    }
}

impl Default for SensorHub {
    fn default() -> Self {
        Self::new()
    }
}
