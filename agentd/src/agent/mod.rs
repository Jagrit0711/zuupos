//! Core ReAct-style agent loop — the "mind" of ZuuposOS.
//!
//! The ZuuposOS agent loop continuously:
//!   1. PERCEIVE  — collect hardware metrics, time-of-day, episodic context
//!   2. REASON    — run LLM (or fallback) with rich system-aware prompt
//!   3. PLAN      — decompose reasoning into multi-step plan
//!   4. ACT       — execute tool calls sequentially
//!   5. REFLECT   — store episode in SQLite memory, update pattern model

use crate::context::ContextEngine;
use crate::ipc::{Emotion, IpcClient, ShellEvent, SystemMetrics};
use crate::llm::LlmEngine;
use crate::planner::Planner;
use crate::sensors::SensorHub;
use crate::tools::{ToolCall, ToolRegistry, ToolResult};
use chrono::Local;
use log::{debug, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct AgentConfig {
    pub model_path: String,
    pub system_prompt: String,
    pub max_tokens: usize,
    pub temperature: f64,
    pub context_window: usize,
}

/// The core AI agent — the "brain" of ZuuposOS.
pub struct Agent {
    config: AgentConfig,
    llm: LlmEngine,
    context: ContextEngine,
    tools: ToolRegistry,
    planner: Planner,
    sensors: SensorHub,
    ipc: IpcClient,
    loop_count: u64,
    last_metrics: Option<SystemMetrics>,
}

impl Agent {
    pub async fn new(config: AgentConfig) -> Self {
        info!(
            "[agent] Initializing agent with model: {}",
            config.model_path
        );
        let llm = LlmEngine::load(&config.model_path, config.temperature).await;
        let context = ContextEngine::new(config.context_window);
        let ipc = IpcClient::new();
        let tools = ToolRegistry::new(ipc.clone());
        let planner = Planner::new(ipc.clone());
        let sensors = SensorHub::new();

        Self {
            config,
            llm,
            context,
            tools,
            planner,
            sensors,
            ipc,
            loop_count: 0,
            last_metrics: None,
        }
    }

    /// The main agent loop — runs forever.
    pub async fn run(&mut self) {
        // Signal online
        self.ipc.send(ShellEvent::AgentReady);
        self.ipc.send(ShellEvent::AvatarEmotion(Emotion::Happy));
        self.ipc
            .send(ShellEvent::StatusUpdate("Zuupos Agent Online".to_string()));

        loop {
            self.loop_count += 1;
            info!("[agent] ═══ Cycle #{} ═══", self.loop_count);

            // ── 1. PERCEIVE ──────────────────────────────────────────────────
            let (perception, metrics) = self.perceive().await;
            self.last_metrics = Some(metrics.clone());

            // Push live metrics to HUD
            self.ipc.send(ShellEvent::Metrics(metrics.clone()));

            debug!("[agent] Perception: {:?}", perception.context_summary);

            // ── 2. REASON ────────────────────────────────────────────────────
            self.ipc.send(ShellEvent::AvatarEmotion(Emotion::Thinking));
            self.ipc.send(ShellEvent::StatusUpdate(format!(
                "Reasoning… cycle #{}",
                self.loop_count
            )));

            let reasoning = self.reason(&perception, &metrics).await;
            info!(
                "[agent] Reasoning output ({} chars): {}…",
                reasoning.len(),
                &reasoning[..reasoning.len().min(80)]
            );

            // ── 3. PLAN ──────────────────────────────────────────────────────
            let tool_calls = self.planner.decompose(&reasoning);
            info!("[agent] Plan: {} steps", tool_calls.len());

            // ── 4. ACT ───────────────────────────────────────────────────────
            self.ipc
                .send(ShellEvent::AvatarEmotion(Emotion::Processing));
            let results = self.act(tool_calls).await;

            // ── 5. REFLECT ───────────────────────────────────────────────────
            self.ipc.send(ShellEvent::AvatarEmotion(Emotion::Idle));
            self.reflect(&perception, &reasoning, &results).await;

            // Adaptive sleep: busier system → faster cycles; idle → slower
            let sleep_ms = self.adaptive_sleep_ms(&metrics);
            debug!("[agent] Sleeping {}ms before next cycle", sleep_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
        }
    }

    // ── PERCEIVE ─────────────────────────────────────────────────────────────

    async fn perceive(&mut self) -> (Perception, SystemMetrics) {
        let metrics = self.sensors.read();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let now = Local::now();
        let context_summary = format!(
            "{} | CPU:{:.0}% Mem:{}/{}MB | Patterns: {}",
            now.format("%H:%M:%S"),
            metrics.cpu_usage,
            metrics.mem_used_mb,
            metrics.mem_total_mb,
            self.context.get_summary()
        );

        let events = self.collect_system_events(&metrics);

        let p = Perception {
            timestamp,
            events,
            context_summary,
        };
        (p, metrics)
    }

    fn collect_system_events(&self, metrics: &SystemMetrics) -> Vec<SystemEvent> {
        let mut events = Vec::new();

        // CPU pressure event
        if metrics.cpu_usage > 75.0 {
            events.push(SystemEvent::HighCpuPressure {
                percent: metrics.cpu_usage,
            });
        }

        // Memory pressure event
        let mem_pct = metrics.mem_used_mb as f32 / metrics.mem_total_mb.max(1) as f32 * 100.0;
        if mem_pct > 80.0 {
            events.push(SystemEvent::HighMemoryPressure {
                used_mb: metrics.mem_used_mb,
            });
        }

        // Time-of-day context
        let hour = Local::now().hour();
        events.push(SystemEvent::TimeOfDay { hour });

        events
    }

    // ── REASON ───────────────────────────────────────────────────────────────

    async fn reason(&mut self, perception: &Perception, metrics: &SystemMetrics) -> String {
        let hour = Local::now().hour();
        let time_context = match hour {
            6..=9 => "Morning startup — user may be starting their day",
            10..=12 => "Mid-morning — focus and productivity window",
            13..=14 => "After lunch — energy dip, may need break reminders",
            15..=17 => "Afternoon — active work period",
            18..=20 => "Evening — wind-down, summary time",
            21..=23 => "Night — rest mode, dim displays",
            _ => "Late night / early morning — minimal intervention",
        };

        let patterns = self.context.get_patterns();
        let pattern_summary = if patterns.is_empty() {
            "No patterns learned yet.".to_string()
        } else {
            patterns
                .iter()
                .map(|p| format!("{}={} (×{})", p.key, p.value, p.frequency))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let prompt = format!(
            r#"[ZUUPOS AGENT CYCLE #{cycle}]
TIME: {time} — {time_ctx}
SYSTEM: CPU {cpu:.1}% | Mem {mem_used}/{mem_total}MB | Load {load:.2}
EVENTS: {events:?}
MEMORY (recent): {context}
PATTERNS (learned): {patterns}

As ZuuposOS, you proactively assist the user without being asked.
Based on the above telemetry and context, decide what to do now.
Output JSON tool calls for each action (one per line, no other text needed):"#,
            cycle = self.loop_count,
            time = Local::now().format("%H:%M:%S"),
            time_ctx = time_context,
            cpu = metrics.cpu_usage,
            mem_used = metrics.mem_used_mb,
            mem_total = metrics.mem_total_mb,
            load = metrics.load_avg_1m,
            events = perception.events,
            context = perception.context_summary,
            patterns = pattern_summary,
        );

        self.llm
            .generate(&self.config.system_prompt, &prompt, self.config.max_tokens)
            .await
    }

    // ── ACT ──────────────────────────────────────────────────────────────────

    async fn act(&mut self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        let mut results = Vec::new();
        for (i, call) in calls.into_iter().enumerate() {
            info!("[agent] ACT step {}: {} {:?}", i + 1, call.tool, call.args);
            self.planner.mark_active(i);
            let result = self.tools.execute(&call).await;
            if result.success {
                self.planner.mark_done(i);
            } else {
                self.planner.mark_failed(i);
                warn!("[agent] Tool failed: {}", result.output);
            }
            results.push(result);
        }
        results
    }

    // ── REFLECT ──────────────────────────────────────────────────────────────

    async fn reflect(&mut self, perception: &Perception, reasoning: &str, results: &[ToolResult]) {
        self.context
            .store_episode(perception, reasoning, results)
            .await;
        info!(
            "[agent] Reflected. Total memory episodes: {}",
            self.context.cycle_count()
        );
    }

    // ── Adaptive sleep ────────────────────────────────────────────────────────

    fn adaptive_sleep_ms(&self, metrics: &SystemMetrics) -> u64 {
        let base = 5000u64; // 5s default
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=800);

        // Faster cycles when CPU/memory pressure is high
        if metrics.cpu_usage > 70.0 || metrics.load_avg_1m > 4.0 {
            return 2000 + jitter;
        }
        // Slower cycles at night
        let hour = Local::now().hour();
        if hour < 6 || hour >= 22 {
            return 15000 + jitter;
        }
        base + jitter
    }
}

// ─── Perception ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Perception {
    pub timestamp: u64,
    pub events: Vec<SystemEvent>,
    pub context_summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SystemEvent {
    KeyPress(u8),
    VoiceInput(String),
    TimerExpired { id: u64 },
    NetworkEvent { event: String },
    TaskReminder { task_id: u64, title: String },
    HighCpuPressure { percent: f32 },
    HighMemoryPressure { used_mb: u64 },
    TimeOfDay { hour: u32 },
}

// helper for hour extraction
trait LocalHour {
    fn hour(self) -> u32;
}
impl LocalHour for chrono::DateTime<chrono::Local> {
    fn hour(self) -> u32 {
        chrono::Timelike::hour(&self)
    }
}
