//! Background LLM worker for ZuuposOS Shell.
//!
//! Handles automatic detection, auto-initiation of Ollama if not running,
//! model discovery, and asynchronous prompt execution so the shell compositor
//! maintains a rock-solid 60 FPS without dropping frames or blocking.

use log::{info, warn};
use serde::Deserialize;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmStatus {
    Ready(String),
    Initiating(String),
    NotInstalled,
    OfflineFallback,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub prompt: String,
    pub text: String,
    pub model: String,
}

pub struct LlmManager {
    prompt_tx: Sender<String>,
    response_rx: Receiver<LlmResponse>,
    status_rx: Receiver<LlmStatus>,
    current_status: LlmStatus,
}

impl Default for LlmManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmManager {
    pub fn new() -> Self {
        let (prompt_tx, prompt_rx) = channel::<String>();
        let (response_tx, response_rx) = channel::<LlmResponse>();
        let (status_tx, status_rx) = channel::<LlmStatus>();

        std::thread::spawn(move || {
            worker_loop(prompt_rx, response_tx, status_tx);
        });

        Self {
            prompt_tx,
            response_rx,
            status_rx,
            current_status: LlmStatus::Initiating("Connecting to LLM...".to_string()),
        }
    }

    pub fn submit(&self, prompt: &str) {
        let _ = self.prompt_tx.send(prompt.to_string());
    }

    pub fn poll_response(&self) -> Option<LlmResponse> {
        self.response_rx.try_recv().ok()
    }

    pub fn poll_status(&mut self) -> &LlmStatus {
        while let Ok(new_status) = self.status_rx.try_recv() {
            self.current_status = new_status;
        }
        &self.current_status
    }

    pub fn status(&self) -> &LlmStatus {
        &self.current_status
    }

    pub fn model_name(&mut self) -> &str {
        match self.poll_status() {
            LlmStatus::Ready(m) => m.as_str(),
            LlmStatus::Initiating(_) => "starting…",
            LlmStatus::NotInstalled => "no ollama",
            LlmStatus::OfflineFallback => "offline",
        }
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Option<Vec<ModelEntry>>,
}

#[derive(Deserialize)]
struct ModelEntry {
    name: String,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

fn worker_loop(
    prompt_rx: Receiver<String>,
    response_tx: Sender<LlmResponse>,
    status_tx: Sender<LlmStatus>,
) {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap_or_default();

    // 1. Check or initiate Ollama
    let mut active_model = check_or_initiate_ollama(&client, &status_tx);

    // 2. Main query loop
    while let Ok(prompt) = prompt_rx.recv() {
        if active_model.is_none() {
            // Re-check in case Ollama was started after boot
            active_model = check_or_initiate_ollama(&client, &status_tx);
        }

        let model = active_model.as_deref().unwrap_or("llama3.2:latest");
        let reply = if let Some(m) = active_model.as_deref() {
            query_ollama(&client, m, &prompt)
        } else {
            autonomous_fallback_reply(&prompt)
        };

        let _ = response_tx.send(LlmResponse {
            prompt,
            text: reply,
            model: model.to_string(),
        });
    }
}

fn check_or_initiate_ollama(
    client: &reqwest::blocking::Client,
    status_tx: &Sender<LlmStatus>,
) -> Option<String> {
    let url = "http://localhost:11434/api/tags";

    // First attempt: is Ollama already running?
    if let Ok(resp) = client.get(url).send() {
        if resp.status().is_success() {
            if let Ok(tags) = resp.json::<TagsResponse>() {
                let model = select_best_model(tags);
                info!("[llm] Ollama detected with model: {}", model);
                let _ = status_tx.send(LlmStatus::Ready(model.clone()));
                return Some(model);
            }
        }
    }

    // Ollama not responding on port 11434 — attempt auto-initiation
    let ollama_bin = find_ollama_binary();
    if let Some(bin) = ollama_bin {
        info!("[llm] Ollama installed at '{}'. Auto-initiating daemon...", bin);
        let _ = status_tx.send(LlmStatus::Initiating("Starting Ollama daemon...".to_string()));

        // Spawn `ollama serve` in the background
        let _ = Command::new(&bin)
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        // Wait up to 3.5 seconds for it to start
        for _ in 0..14 {
            std::thread::sleep(Duration::from_millis(250));
            if let Ok(resp) = client.get(url).send() {
                if resp.status().is_success() {
                    if let Ok(tags) = resp.json::<TagsResponse>() {
                        let model = select_best_model(tags);
                        info!("[llm] Ollama auto-initiated successfully with model: {}", model);
                        let _ = status_tx.send(LlmStatus::Ready(model.clone()));
                        return Some(model);
                    }
                }
            }
        }
        warn!("[llm] Ollama daemon spawn timed out");
    } else {
        warn!("[llm] Ollama is not installed on this system");
        let _ = status_tx.send(LlmStatus::NotInstalled);
    }

    let _ = status_tx.send(LlmStatus::OfflineFallback);
    None
}

fn find_ollama_binary() -> Option<String> {
    for candidate in &[
        "ollama",
        "/opt/homebrew/bin/ollama",
        "/usr/local/bin/ollama",
        "/usr/bin/ollama",
    ] {
        if let Ok(status) = Command::new(candidate).arg("--version").output() {
            if status.status.success() {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

fn select_best_model(tags: TagsResponse) -> String {
    let models = tags.models.unwrap_or_default();
    let names: Vec<String> = models.into_iter().map(|m| m.name).collect();

    // Preference hierarchy
    for preferred in &[
        "llama3.2:latest",
        "llama3.2:3b",
        "llama3.2",
        "llama3.1:latest",
        "phi4-mini",
        "phi3:latest",
        "mistral:latest",
        "qwen2.5:latest",
    ] {
        if let Some(found) = names.iter().find(|n| n.starts_with(preferred)) {
            return found.clone();
        }
    }

    names.first().cloned().unwrap_or_else(|| "llama3.2:latest".to_string())
}

fn query_ollama(client: &reqwest::blocking::Client, model: &str, prompt: &str) -> String {
    let url = "http://localhost:11434/api/generate";
    let system = "You are Zuupos, an AI-native operating system. Answer concisely, helpfully, and with warmth. Limit responses to 2-3 sentences.";

    let req_body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "system": system,
        "stream": false,
        "options": {
            "temperature": 0.65,
            "num_predict": 128
        }
    });

    match client.post(url).json(&req_body).send() {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(gen) = resp.json::<GenerateResponse>() {
                let trimmed = gen.response.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
            "I heard you! Operating system services are nominal.".to_string()
        }
        _ => autonomous_fallback_reply(prompt),
    }
}

fn autonomous_fallback_reply(prompt: &str) -> String {
    let lower = prompt.trim().to_lowercase();
    if lower.contains("hi") || lower.contains("hello") || lower.contains("hey") {
        "Hey! I'm Zuupos, your AI operating system. How can I help you today?".to_string()
    } else if lower.contains("who are you") || lower.contains("what are you") {
        "I am ZuuposOS — an AI-native operating system built in Rust. I reason, plan, and execute tasks autonomously.".to_string()
    } else if lower.contains("how are you") {
        "Running smoothly at 60 FPS! All kernel subsystems and microkernel IPC channels are nominal.".to_string()
    } else {
        format!(
            "Understood: \"{}\". Ready to dispatch agents and manage tasks across the system.",
            prompt.trim()
        )
    }
}
