//! ZuuposOS Agentic Runtime Daemon (agentd)
//!
//! This is PID 1 of ZuuposOS — the AI agent that *is* the OS.
//! It runs the core PERCEIVE → REASON → ACT → REFLECT loop.

mod agent;
mod context;
mod ipc;
mod llm;
mod tools;
mod voice;

use agent::{Agent, AgentConfig};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("╔══════════════════════════════════════════════╗");
    info!("║        ZuuposOS Agentic Runtime v0.1         ║");
    info!("║  PERCEIVE → REASON → ACT → REFLECT → loop   ║");
    info!("╚══════════════════════════════════════════════╝");

    let config = AgentConfig {
        model_path: std::env::var("ZUUPOS_MODEL_PATH")
            .unwrap_or_else(|_| "models/phi-4-mini.gguf".to_string()),
        system_prompt: include_str!("agent/system_prompt.txt").to_string(),
        max_tokens: 2048,
        temperature: 0.7,
        context_window: 8192,
    };

    let mut agent = Agent::new(config).await;
    info!("[agentd] Agent initialized. Starting main loop...");

    agent.run().await;
}
