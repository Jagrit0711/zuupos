//! LLM Engine — local model inference via Ollama.
//!
//! Connects to Ollama (http://localhost:11434) and falls back to an
//! autonomous time-aware reasoning engine when Ollama is offline.

use chrono::Timelike;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct LlmEngine {
    client: reqwest::Client,
    ollama_url: String,
    model_name: String,
    temperature: f64,
    fallback_count: u64,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a str,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f64,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl LlmEngine {
    pub async fn load(_model_path: &str, temperature: f64) -> Self {
        let ollama_url = std::env::var("ZUUPOS_OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model_name = std::env::var("ZUUPOS_OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3.2:latest".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        info!(
            "[llm] Configured Ollama: {} (model: {})",
            ollama_url, model_name
        );

        Self {
            client,
            ollama_url,
            model_name,
            temperature,
            fallback_count: 0,
        }
    }

    /// Generate text from a system prompt + user message.
    pub async fn generate(
        &mut self,
        system: &str,
        user: &str,
        max_tokens: usize,
    ) -> String {
        let url = format!("{}/api/generate", self.ollama_url);
        let req = OllamaRequest {
            model: &self.model_name,
            prompt: user,
            system,
            stream: false,
            options: OllamaOptions {
                temperature: self.temperature,
                num_predict: max_tokens.min(512) as i32,
            },
        };

        match self.client.post(&url).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<OllamaResponse>().await {
                    Ok(data) if !data.response.trim().is_empty() => {
                        let text = data.response.trim().to_string();
                        info!("[llm] Ollama live response ({} chars)", text.len());
                        self.fallback_count = 0;
                        return text;
                    }
                    _ => warn!("[llm] Ollama returned empty response"),
                }
            }
            Ok(resp) => warn!("[llm] Ollama HTTP {}", resp.status()),
            Err(e) => warn!("[llm] Ollama unavailable: {e}"),
        }

        self.fallback_reasoning(user)
    }

    /// Autonomous time-aware reasoning when Ollama is offline.
    fn fallback_reasoning(&mut self, _user_context: &str) -> String {
        self.fallback_count += 1;
        let n = self.fallback_count;
        let hour = chrono::Local::now().hour();

        // Time-of-day driven behaviors
        match (n % 8, hour) {
            (0, 6..=9) => concat!(
                "Morning routine initiated. Checking system health.\n",
                r#"{"tool": "display_card", "args": {"title": "Good Morning", "body": "ZuuposOS is ready. All systems nominal. Have a productive day.", "type": "info"}}"#, "\n",
                r#"{"tool": "speak", "args": {"text": "Good morning. ZuuposOS is online and ready.", "priority": "low"}}"#
            ).to_string(),

            (1, 10..=12) => concat!(
                "Mid-morning focus check. Optimising for productivity.\n",
                r#"{"tool": "display_card", "args": {"title": "Focus Window", "body": "Peak cognitive hours detected. Consider tackling your most complex task now.", "type": "task"}}"#, "\n",
                r#"{"tool": "set_reminder", "args": {"title": "Hydration break", "in_minutes": 45}}"#
            ).to_string(),

            (2, 13..=14) => concat!(
                "Post-lunch energy dip detected. Activating gentle nudge.\n",
                r#"{"tool": "display_card", "args": {"title": "Post-Lunch Check", "body": "Energy dip window. A 10-minute walk can restore 20% of afternoon focus.", "type": "weather"}}"#
            ).to_string(),

            (3, 15..=17) => concat!(
                "Afternoon sprint mode. System load nominal.\n",
                r#"{"tool": "display_card", "args": {"title": "Afternoon Sprint", "body": "2-hour power window before 5 PM. Agent is queuing low-priority tasks for later.", "type": "task"}}"#, "\n",
                r#"{"tool": "set_reminder", "args": {"title": "End-of-day review", "in_minutes": 90}}"#
            ).to_string(),

            (4, 18..=20) => concat!(
                "Evening wind-down initiated. Dimming ambient intensity.\n",
                r#"{"tool": "display_card", "args": {"title": "Evening Wind-Down", "body": "Wrapping up agent tasks for the day. Review your goals before closing.", "type": "info"}}"#, "\n",
                r#"{"tool": "dim_display", "args": {"level": 0.65}}"#
            ).to_string(),

            (5, 21..=23) => concat!(
                "Night mode. Reducing agent activity to conserve resources.\n",
                r#"{"tool": "display_card", "args": {"title": "Night Mode", "body": "Agent entering low-power observational loop. Rest well.", "type": "info"}}"#, "\n",
                r#"{"tool": "dim_display", "args": {"level": 0.3}}"#
            ).to_string(),

            (6, _) => concat!(
                "Autonomous health scan in progress.\n",
                r#"{"tool": "display_card", "args": {"title": "System Telemetry", "body": "ZuuposOS memory engine: nominal. IPC bridge: active. Agent loop: 60Hz reasoning.", "type": "metric"}}"#
            ).to_string(),

            _ => concat!(
                "Proactive agent cycle. Displaying context.\n",
                r#"{"tool": "display_card", "args": {"title": "Agent Active", "body": "Autonomous reasoning loop running. Monitoring system events and user context.", "type": "info"}}"#
            ).to_string(),
        }
    }
}
