//! IPC client for agentd — connects to ZuuposOS shell over Unix domain socket.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/zuupos_shell.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    Idle,
    Thinking,
    Speaking,
    Alert,
    Happy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Info,
    Task,
    Alert,
    Weather,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub card_type: CardType,
    pub age_ticks: u32,
    pub max_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellEvent {
    Card(Card),
    AvatarEmotion(Emotion),
    StatusUpdate(String),
}

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
        // If not connected, try reconnecting
        if guard.is_none() {
            *guard = UnixStream::connect(DEFAULT_SOCKET_PATH).ok();
        }

        if let Some(stream) = guard.as_mut() {
            if let Ok(mut json) = serde_json::to_string(&event) {
                json.push('\n');
                if stream.write_all(json.as_bytes()).is_err() {
                    *guard = None; // Socket broke, clear for reconnection next time
                } else {
                    let _ = stream.flush();
                }
            }
        }
    }
}
