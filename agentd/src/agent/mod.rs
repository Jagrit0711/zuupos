//! Core ReAct-style agent loop.
//!
//! The ZuuposOS agent loop is the central "mind" of the operating system.
//! It continuously:
//!   1. PERCEIVE  — collect events from hardware, sensors, user input, system state
//!   2. REASON    — run the LLM to decide what action(s) to take
//!   3. ACT       — invoke tools (file system, process spawn, display, network)
//!   4. REFLECT   — store outcome in long-term memory, update context

use crate::context::ContextEngine;
use crate::llm::LlmEngine;
use crate::tools::{ToolRegistry, ToolCall, ToolResult};
use log::{info, warn, debug};
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
    loop_count: u64,
}

impl Agent {
    pub async fn new(config: AgentConfig) -> Self {
        info!("[agent] Loading LLM from: {}", config.model_path);
        let llm = LlmEngine::load(&config.model_path, config.temperature).await;
        let context = ContextEngine::new(config.context_window);
        let tools = ToolRegistry::default();

        Self {
            config,
            llm,
            context,
            tools,
            loop_count: 0,
        }
    }

    /// The main agent loop — runs forever.
    pub async fn run(&mut self) {
        loop {
            self.loop_count += 1;
            debug!("[agent] Loop #{}", self.loop_count);

            // === PERCEIVE ===
            let perception = self.perceive().await;
            debug!("[agent] Perception: {:?}", perception);

            // === REASON ===
            let reasoning = self.reason(&perception).await;
            debug!("[agent] Reasoning output length: {} chars", reasoning.len());

            // === ACT ===
            let actions = self.parse_tool_calls(&reasoning);
            let results = self.act(actions).await;

            // === REFLECT ===
            self.reflect(&perception, &reasoning, &results).await;

            // Yield to avoid busy-looping on idle (in real OS: wait for event)
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Collect system events and observations.
    async fn perceive(&self) -> Perception {
        let mut events = Vec::new();

        // Check keyboard input
        // TODO: In real OS, poll from kernel keyboard driver via IPC
        // events.extend(keyboard::poll());

        // Check system time
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Check active tasks, reminders, calendar
        // TODO: query context engine for upcoming deadlines

        Perception {
            timestamp,
            events,
            context_summary: self.context.get_summary(),
        }
    }

    /// Run the LLM with current perception to produce a reasoning trace.
    async fn reason(&mut self, perception: &Perception) -> String {
        let prompt = format!(
            "SYSTEM_TIME: {}\nCONTEXT: {}\nEVENTS: {:?}\n\nWhat should I do next?",
            perception.timestamp,
            perception.context_summary,
            perception.events,
        );
        self.llm.generate(&self.config.system_prompt, &prompt, self.config.max_tokens).await
    }

    /// Parse LLM output for structured tool calls (JSON function calls).
    fn parse_tool_calls(&self, llm_output: &str) -> Vec<ToolCall> {
        // The LLM is prompted to output JSON tool calls in a structured format:
        // {"tool": "display_card", "args": {"title": "...", "body": "..."}}
        // We extract all such JSON objects from the output.

        let mut calls = Vec::new();
        let mut start = 0;
        while let Some(pos) = llm_output[start..].find("{\"tool\"") {
            let abs_pos = start + pos;
            if let Some(end) = llm_output[abs_pos..].find('\n') {
                let json_str = &llm_output[abs_pos..abs_pos + end];
                if let Ok(call) = serde_json::from_str::<ToolCall>(json_str) {
                    calls.push(call);
                }
                start = abs_pos + end + 1;
            } else {
                break;
            }
        }
        calls
    }

    /// Execute tool calls and collect results.
    async fn act(&mut self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        let mut results = Vec::new();
        for call in calls {
            info!("[agent] ACT: {} {:?}", call.tool, call.args);
            let result = self.tools.execute(&call).await;
            results.push(result);
        }
        results
    }

    /// Store the episode in long-term memory.
    async fn reflect(&mut self, perception: &Perception, reasoning: &str, results: &[ToolResult]) {
        self.context.store_episode(perception, reasoning, results).await;
    }
}

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
}
