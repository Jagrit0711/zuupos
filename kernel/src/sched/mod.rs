//! Work-stealing multi-core scheduler.
//! Phase 1.3: Basic round-robin on single CPU.

use alloc::collections::VecDeque;
use spinning_top::Spinlock;

pub static TICK_COUNT: Spinlock<u64> = Spinlock::new(0);

pub fn init() {
    log::debug!("[sched] Scheduler initialized (Phase 1.3 stub: round-robin)");
}

pub fn tick() {
    *TICK_COUNT.lock() += 1;
    // TODO Phase 1.3: context switch to next runnable process
}

pub fn spawn_init_process() {
    log::info!("[sched] Spawning PID 1 (zuupos-agentd) — Phase 1.5 will load ELF");
    // TODO Phase 1.5: Load agentd ELF from filesystem, create process context
    //                 and schedule it. For now, call a stub.
    run_agent_stub();
}

/// Temporary agent stub that runs until Phase 1.5.
fn run_agent_stub() {
    log::info!("[agentd-stub] ZuuposOS agent loop starting...");
    log::info!("[agentd-stub] PERCEIVE → REASON → ACT → REFLECT");
    log::info!("[agentd-stub] Waiting for Phase 1.5 (LLM integration)...");
    // In real impl: call into agentd binary
}
