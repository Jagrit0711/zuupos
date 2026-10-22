//! Voice subsystem — STT via whisper.cpp FFI, TTS via espeak-ng or kokoro.
//! Phase 1.8.

pub struct VoiceEngine {
    // whisper_ctx: WhisperContext,  // Phase 1.8
}

impl VoiceEngine {
    pub fn init() -> Self {
        log::info!("[voice] Voice engine stub (Phase 1.8)");
        Self {}
    }

    pub fn listen(&self) -> Option<String> {
        // TODO Phase 1.8: record from mic, run whisper inference
        None
    }

    pub fn speak(&self, text: &str) {
        // TODO Phase 1.8: run TTS (espeak-ng or kokoro)
        log::info!("[voice::tts] Would say: \"{}\"", text);
    }
}
