//! IPC receiver for shell — receives messages from agentd.
//!
//! The shell listens on a Unix domain socket and deserializes ShellEvent
//! messages sent by agentd. Events are queued for the render loop to consume.

use crate::avatar::Emotion;
use crate::cards::Card;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/zuupos_shell.sock";

// ─── Shared IPC types (mirrors agentd/src/ipc.rs) ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub battery_pct: Option<f32>,
    pub load_avg_1m: f32,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockEntry {
    pub id: String,
    pub label: String,
    pub icon_char: char,
    pub active: bool,
    pub pulse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellEvent {
    Card(Card),
    AvatarEmotion(Emotion),
    StatusUpdate(String),
    Metrics(SystemMetrics),
    TaskPlan(Vec<PlanStep>),
    DockUpdate(DockEntry),
    Notification { title: String, body: String, urgency: u8 },
    BootStarted,
    AgentReady,
}

// ─── IPC Receiver ─────────────────────────────────────────────────────────────

pub struct IpcReceiver {
    socket_path: String,
    queue: Arc<Mutex<Vec<ShellEvent>>>,
}

impl IpcReceiver {
    pub fn new() -> Self {
        Self {
            socket_path: DEFAULT_SOCKET_PATH.to_string(),
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn listen(&self) {
        let path = Path::new(&self.socket_path);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }

        let listener = match UnixListener::bind(path) {
            Ok(l) => {
                info!("[ipc] Shell socket listening on {}", self.socket_path);
                l
            }
            Err(e) => {
                error!("[ipc] Failed to bind Unix socket: {}", e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let queue = self.queue.clone();
                    std::thread::spawn(move || handle_client(stream, queue));
                }
                Err(e) => warn!("[ipc] Accept error: {}", e),
            }
        }
    }

    pub fn drain_events(&mut self) -> Vec<ShellEvent> {
        let mut q = self.queue.lock().unwrap();
        std::mem::take(&mut *q)
    }
}

fn handle_client(stream: UnixStream, queue: Arc<Mutex<Vec<ShellEvent>>>) {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ShellEvent>(trimmed) {
                    Ok(event) => {
                        queue.lock().unwrap().push(event);
                    }
                    Err(e) => {
                        warn!("[ipc] Parse error: {} | line: {}", e, &trimmed[..trimmed.len().min(60)]);
                    }
                }
            }
            Err(_) => break,
        }
    }
}
