//! LLM Engine — local model inference.
//!
//! Uses HuggingFace `candle` (pure Rust) for smaller models (Phi-4, Gemma)
//! or llama.cpp via FFI for larger GGUF models (Llama 3, Mistral).
//!
//! Phase 1.5 target: load and run a quantized Phi-4 Mini (4B params, ~2.5 GB)
//! that comfortably fits in RAM on most machines.

use log::{info, warn};

pub struct LlmEngine {
    model_path: String,
    temperature: f64,
    // Phase 1.5: candle model weights + tokenizer loaded here
    // model: Option<PhiModel>,
    // tokenizer: Option<Tokenizer>,
}

impl LlmEngine {
    pub async fn load(model_path: &str, temperature: f64) -> Self {
        info!("[llm] Loading model from: {}", model_path);

        // Phase 1.5: Actual loading:
        // let tokenizer = Tokenizer::from_file(format!("{}/tokenizer.json", model_path));
        // let model = PhiModel::load_gguf(model_path, device)?;

        if !std::path::Path::new(model_path).exists() {
            warn!(
                "[llm] Model not found at '{}'. Running in stub mode.",
                model_path
            );
            warn!("[llm] Download a model with: `zuupos model pull phi-4-mini`");
        }

        Self {
            model_path: model_path.to_string(),
            temperature,
        }
    }

    /// Generate text from a system prompt + user message.
    pub async fn generate(&mut self, system: &str, user: &str, max_tokens: usize) -> String {
        // Phase 1.5: Real inference:
        // let tokens = self.tokenizer.encode(format!("<|system|>{}<|user|>{}<|assistant|>", system, user));
        // let output = self.model.forward(&tokens, max_tokens, self.temperature);
        // return self.tokenizer.decode(output);

        // Stub: return a canned response that demonstrates the tool call format
        let stub = format!(
            r#"Observing system context. System time: now. No urgent events.
I will display a status card to show the OS is alive.
{{"tool": "display_card", "args": {{"title": "ZuuposOS Active", "body": "Agent loop running. LLM stub mode — Phase 1.5 will load real model from: {}", "type": "info"}}}}
{{"tool": "speak", "args": {{"text": "ZuuposOS is online and thinking.", "priority": "low"}}}}"#,
            self.model_path
        );
        stub
    }
}
