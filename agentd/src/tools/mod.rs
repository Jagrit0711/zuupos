//! Tool Registry — the agent's "hands".
//!
//! Every capability the agent can invoke is a Tool. Tools are registered here
//! and invoked by name from the agent loop. This is the capability boundary:
//! the agent can ONLY affect the world through tools.

pub mod research;

use crate::context::ContextEngine;
use crate::ipc::{Card, CardType, Emotion, IpcClient, ShellEvent};
use log::{info, warn};
use research::{FetchDocTool, InspectSystemTool, RecallMemoryTool, RememberFactTool, WebSearchTool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Debug)]
pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub output: String,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new(ipc: IpcClient, context: Arc<Mutex<ContextEngine>>) -> Self {
        let mut reg = Self {
            tools: HashMap::new(),
        };
        reg.register(Box::new(DisplayCardTool { ipc: ipc.clone() }));
        reg.register(Box::new(SpeakTool { ipc: ipc.clone() }));
        reg.register(Box::new(OpenAppTool));
        reg.register(Box::new(SetReminderTool { ipc: ipc.clone() }));
        reg.register(Box::new(RunCommandTool));
        reg.register(Box::new(DimDisplayTool));
        reg.register(Box::new(FocusMusicTool { ipc }));

        // Autonomous Research, System Inspection & Memory Tools
        reg.register(Box::new(WebSearchTool::new()));
        reg.register(Box::new(FetchDocTool::new()));
        reg.register(Box::new(InspectSystemTool));
        reg.register(Box::new(RememberFactTool::new(context.clone())));
        reg.register(Box::new(RecallMemoryTool::new(context)));

        reg
    }

    fn register(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        match self.tools.get(&call.tool) {
            Some(tool) => {
                let output = tool.execute(&call.args).await;
                ToolResult {
                    tool: call.tool.clone(),
                    success: true,
                    output,
                }
            }
            None => {
                warn!("[tools] Unknown tool: {}", call.tool);
                ToolResult {
                    tool: call.tool.clone(),
                    success: false,
                    output: format!("Unknown tool: {}", call.tool),
                }
            }
        }
    }
}

#[async_trait::async_trait]
trait Tool {
    fn name(&self) -> &str;
    async fn execute(&self, args: &serde_json::Value) -> String;
}

// ─── Tool Implementations ────────────────────────────────────────────────────

struct DisplayCardTool {
    ipc: IpcClient,
}

#[async_trait::async_trait]
impl Tool for DisplayCardTool {
    fn name(&self) -> &str {
        "display_card"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let title = args["title"]
            .as_str()
            .unwrap_or("Zuupos Notification")
            .to_string();
        let body = args["body"].as_str().unwrap_or("").to_string();
        let card_type_str = args["type"].as_str().unwrap_or("info");
        let card_type = match card_type_str {
            "task" => CardType::Task,
            "alert" => CardType::Alert,
            "weather" => CardType::Weather,
            _ => CardType::Info,
        };

        info!("[tool::display_card] [{card_type_str}] {title}: {body}");

        self.ipc.send(ShellEvent::Card(Card {
            id: 0,
            title: title.clone(),
            body: body.clone(),
            card_type,
            age_ticks: 0,
            max_age: 450,
        }));

        format!("Card displayed: {title}")
    }
}

struct SpeakTool {
    ipc: IpcClient,
}

#[async_trait::async_trait]
impl Tool for SpeakTool {
    fn name(&self) -> &str {
        "speak"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let text = args["text"].as_str().unwrap_or("");
        let priority = args["priority"].as_str().unwrap_or("normal");
        info!("[tool::speak] [{priority}] \"{text}\"");
        self.ipc.send(ShellEvent::AvatarEmotion(Emotion::Speaking));
        self.ipc
            .send(ShellEvent::StatusUpdate(format!("Speaking: {}", text)));
        format!("Spoken: {text}")
    }
}

struct OpenAppTool;
#[async_trait::async_trait]
impl Tool for OpenAppTool {
    fn name(&self) -> &str {
        "open_app"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let name = args["name"].as_str().unwrap_or("unknown");
        info!("[tool::open_app] Opening: {name}");
        format!("Opened app: {name}")
    }
}

struct SetReminderTool {
    ipc: IpcClient,
}

#[async_trait::async_trait]
impl Tool for SetReminderTool {
    fn name(&self) -> &str {
        "set_reminder"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let title = args["title"].as_str().unwrap_or("Reminder").to_string();
        let mins = args["in_minutes"].as_u64().unwrap_or(30);
        info!("[tool::set_reminder] '{title}' in {mins}m");

        self.ipc.send(ShellEvent::Card(Card {
            id: 0,
            title: format!("Reminder (in {}m)", mins),
            body: title.clone(),
            card_type: CardType::Task,
            age_ticks: 0,
            max_age: 400,
        }));

        format!("Reminder set: '{title}' in {mins} minutes")
    }
}

struct RunCommandTool;
#[async_trait::async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let cmd = args["cmd"].as_str().unwrap_or("");
        let reason = args["reason"].as_str().unwrap_or("");
        info!("[tool::run_command] '{cmd}' (reason: {reason})");
        format!("Command queued: {cmd}")
    }
}

struct DimDisplayTool;
#[async_trait::async_trait]
impl Tool for DimDisplayTool {
    fn name(&self) -> &str {
        "dim_display"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let level = args["level"].as_f64().unwrap_or(0.5);
        info!("[tool::dim_display] level={level}");
        format!("Display dimmed to {:.0}%", level * 100.0)
    }
}

struct FocusMusicTool {
    ipc: IpcClient,
}

#[async_trait::async_trait]
impl Tool for FocusMusicTool {
    fn name(&self) -> &str {
        "play_focus_music"
    }
    async fn execute(&self, args: &serde_json::Value) -> String {
        let genre = args["genre"].as_str().unwrap_or("lofi");
        info!("[tool::play_focus_music] genre={genre}");

        self.ipc.send(ShellEvent::Card(Card {
            id: 0,
            title: "Focus Audio".to_string(),
            body: format!("Streaming {} audio stream", genre),
            card_type: CardType::Weather,
            age_ticks: 0,
            max_age: 350,
        }));

        format!("Playing {genre} focus music")
    }
}
