//! ZuuposOS Agentic Runtime Daemon (agentd)
//!
//! This is PID 1 of ZuuposOS — the AI agent that *is* the OS.
//! It runs the 5-stage PERCEIVE → REASON → PLAN → ACT → REFLECT loop,
//! backed by real system sensors, a SQLite memory store, and Ollama LLM.

mod agent;
mod context;
mod ipc;
mod llm;
mod planner;
mod sensors;
mod tools;
mod voice;

use agent::{Agent, AgentConfig};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("╔══════════════════════════════════════════════════════╗");
    info!("║          ZuuposOS Agentic Runtime Daemon v0.2        ║");
    info!("║   PERCEIVE → REASON → PLAN → ACT → REFLECT → loop   ║");
    info!("║   Sensors: sysinfo | Memory: SQLite | LLM: Ollama    ║");
    info!("╚══════════════════════════════════════════════════════╝");

    let config = AgentConfig {
        model_path: std::env::var("ZUUPOS_MODEL_PATH")
            .unwrap_or_else(|_| "models/phi-4-mini.gguf".to_string()),
        system_prompt: include_str!("agent/system_prompt.txt").to_string(),
        max_tokens: 512,
        temperature: 0.65,
        context_window: 50,
    };

    let mut agent = Agent::new(config).await;
    info!("[agentd] Agent initialized — starting proactive loop...");

    agent.run().await;
}
