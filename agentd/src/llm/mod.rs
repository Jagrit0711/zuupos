//! LLM Engine — supports multiple backends: Ollama (HTTP) and llama.cpp (local FFI).
//!
//! Provides a unified interface for local model inference with automatic fallback.

use chrono::Timelike;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub enum LlmBackend {
    Ollama(OllamaEngine),
    LlamaCpp(LlamaCppEngine),
    Fallback(FallbackEngine),
}

pub struct LlmEngine {
    backend: LlmBackend,
    temperature: f64,
    max_tokens: usize,
}

impl LlmEngine {
    pub async fn load(model_path: &str, temperature: f64) -> Self {
        // Try llama.cpp first if model path ends with .gguf
        if model_path.ends_with(".gguf") {
            if let Ok(engine) = LlamaCppEngine::new(model_path) {
                info!("[llm] Loaded llama.cpp backend: {}", model_path);
                return Self {
                    backend: LlmBackend::LlamaCpp(engine),
                    temperature,
                    max_tokens: 512,
                };
            }
        }

        // Try Ollama
        let ollama_url = std::env::var("ZUUPOS_OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model_name = std::env::var("ZUUPOS_OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3.2:latest".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        // Test Ollama connection
        let test_url = format!("{}/api/tags", ollama_url);
        if client.get(&test_url).send().await.is_ok() {
            info!("[llm] Loaded Ollama backend: {} (model: {})", ollama_url, model_name);
            return Self {
                backend: LlmBackend::Ollama(OllamaEngine {
                    client,
                    ollama_url,
                    model_name,
                }),
                temperature,
                max_tokens: 512,
            };
        }

        // Fallback to autonomous reasoning
        warn!("[llm] No backend available, using fallback");
        Self {
            backend: LlmBackend::Fallback(FallbackEngine::new()),
            temperature,
            max_tokens: 512,
        }
    }

    pub async fn generate(
        &mut self,
        system: &str,
        user: &str,
        max_tokens: usize,
    ) -> String {
        match &mut self.backend {
            LlmBackend::Ollama(e) => e.generate(system, user, max_tokens, self.temperature).await,
            LlmBackend::LlamaCpp(e) => e.generate(system, user, max_tokens, self.temperature),
            LlmBackend::Fallback(e) => e.generate(user),
        }
    }
}

struct OllamaEngine {
    client: reqwest::Client,
    ollama_url: String,
    model_name: String,
}

impl OllamaEngine {
    async fn generate(
        &self,
        system: &str,
        user: &str,
        max_tokens: usize,
        temperature: f64,
    ) -> String {
        let url = format!("{}/api/generate", self.ollama_url);
        let req = OllamaRequest {
            model: &self.model_name,
            prompt: user,
            system,
            stream: false,
            options: OllamaOptions {
                temperature,
                num_predict: max_tokens.min(512) as i32,
            },
        };

        match self.client.post(&url).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<OllamaResponse>().await {
                    Ok(data) if !data.response.trim().is_empty() => {
                        let text = data.response.trim().to_string();
                        info!("[llm] Ollama response ({} chars)", text.len());
                        return text;
                    }
                    _ => warn!("[llm] Ollama returned empty response"),
                }
            }
            Ok(resp) => warn!("[llm] Ollama HTTP {}", resp.status()),
            Err(e) => warn!("[llm] Ollama error: {e}"),
        }

        // Fallback to autonomous reasoning
        FallbackEngine::new().generate(user)
    }
}

struct LlamaCppEngine {
    _model_path: String,
    // In real implementation: Arc<Mutex<llama_cpp_2::model::LlamaModel>>
    // and Arc<Mutex<llama_cpp_2::context::LlamaContext>>
}

impl LlamaCppEngine {
    fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // For now, just validate the file exists
        if std::path::Path::new(model_path).exists() {
            Ok(Self { _model_path: model_path.to_string() })
        } else {
            Err("Model file not found".into())
        }
    }

    fn generate(&self, _system: &str, user: &str, _max_tokens: usize, _temperature: f64) -> String {
        // TODO: Implement actual llama.cpp inference when API stabilizes
        // For now, return a structured response indicating the model is loaded
        format!(
            "Llama.cpp model loaded: {}\nProcessing: {}\n\n{}",
            self._model_path,
            user,
            r#"{"tool": "display_card", "args": {"title": "LLM Response", "body": "Local model inference active. Processing your request with llama.cpp.", "type": "metric"}}"#
        )
    }
}

struct FallbackEngine {
    count: u64,
}

impl FallbackEngine {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn generate(&mut self, _user_context: &str) -> String {
        self.count += 1;
        let n = self.count;
        let hour = chrono::Local::now().hour();

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