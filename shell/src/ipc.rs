//! IPC receiver for shell — receives messages from agentd.

use crate::avatar::Emotion;
use crate::cards::Card;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/zuupos_shell.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellEvent {
    Card(Card),
    AvatarEmotion(Emotion),
    StatusUpdate(String),
}

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
                info!("[ipc] Listening on {}", self.socket_path);
                l
            }
            Err(e) => {
                error!("[ipc] Failed to bind Unix socket at {}: {}", self.socket_path, e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let queue = self.queue.clone();
                    std::thread::spawn(move || {
                        handle_client(stream, queue);
                    });
                }
                Err(e) => {
                    warn!("[ipc] Accept error: {}", e);
                }
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
                        let mut q = queue.lock().unwrap();
                        q.push(event);
                    }
                    Err(e) => {
                        warn!("[ipc] JSON parse error: {} on line: {}", e, trimmed);
                    }
                }
            }
            Err(_) => break,
        }
    }
}
