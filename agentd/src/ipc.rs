//! IPC protocol — shared types between agentd and shell.
//!
//! All messages are newline-delimited JSON over a Unix domain socket.
//! Socket path: /tmp/zuupos_shell.sock
//!
//! Direction: agentd → shell (currently one-way; bidirectional in Phase 2)

use serde::{Deserialize, Serialize};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/zuupos_shell.sock";

// ─── Enums ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    Idle,
    Thinking,
    Speaking,
    Alert,
    Happy,
    Processing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Info,
    Task,
    Alert,
    Weather,
    Metric,
    Plan,
}

// ─── Data Structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub card_type: CardType,
    pub age_ticks: u32,
    pub max_age: u32,
}

/// A single step in an agent-generated task plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub index: u32,
    pub description: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Active,
    Done,
    Failed,
}

/// Real-time system hardware metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,       // 0..100 percent
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub battery_pct: Option<f32>,
    pub load_avg_1m: f32,
}

/// A dock icon/applet entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockEntry {
    pub id: String,
    pub label: String,
    pub icon_char: char,      // Unicode icon character
    pub active: bool,
    pub pulse: bool,
}

// ─── Shell Events (agentd → shell) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellEvent {
    /// Display a floating glassmorphism card.
    Card(Card),
    /// Change the avatar emotion state.
    AvatarEmotion(Emotion),
    /// Update the status bar text.
    StatusUpdate(String),
    /// Push live system metrics for the HUD.
    Metrics(SystemMetrics),
    /// Show a multi-step plan in the shell.
    TaskPlan(Vec<PlanStep>),
    /// Update a dock entry.
    DockUpdate(DockEntry),
    /// Notification with optional urgency level (0=low, 1=normal, 2=critical).
    Notification { title: String, body: String, urgency: u8 },
    /// Signal to the shell to enter a boot splash.
    BootStarted,
    /// Signal that the agent is fully initialised.
    AgentReady,
}

// ─── IPC Client (used by agentd) ─────────────────────────────────────────────

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct IpcClient {
    stream: Arc<Mutex<Option<UnixStream>>>,
}

impl IpcClient {
    pub fn new() -> Self {
        let stream = UnixStream::connect(DEFAULT_SOCKET_PATH).ok();
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub fn send(&self, event: ShellEvent) {
        let mut guard = self.stream.lock().unwrap();
        // Auto-reconnect
        if guard.is_none() {
            *guard = UnixStream::connect(DEFAULT_SOCKET_PATH).ok();
        }

        if let Some(stream) = guard.as_mut() {
            if let Ok(mut json) = serde_json::to_string(&event) {
                json.push('\n');
                if stream.write_all(json.as_bytes()).is_err() {
                    *guard = None; // Socket broke; retry next cycle
                } else {
                    let _ = stream.flush();
                }
            }
        }
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}
