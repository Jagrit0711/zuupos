//! Avatar — the "face" of ZuuposOS.
//!
//! The avatar lives in the bottom-left corner. Its emotion state is driven
//! by the agentd via IPC, making it visually represent what the AI is doing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Emotion {
    #[default]
    Idle,
    Thinking,
    Speaking,
    Alert,
    Happy,
}

pub struct Avatar {
    emotion: Emotion,
    tick: u64,
    blink_tick: u64,
    next_blink: u64,
    emotion_since: u64,
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            emotion: Emotion::Idle,
            tick: 0,
            blink_tick: 0,
            next_blink: 180,
            emotion_since: 0,
        }
    }

    pub fn set_emotion(&mut self, emotion: Emotion) {
        if self.emotion != emotion {
            self.emotion = emotion;
            self.emotion_since = self.tick;
        }
    }

    pub fn tick(&mut self, global_tick: u64) {
        self.tick = global_tick;

        // Blink logic (suppress blinking while thinking)
        if self.emotion != Emotion::Thinking {
            if global_tick >= self.next_blink {
                self.blink_tick = global_tick;
                // Next blink: 80–280 frames (~1.3–4.6s @ 60fps)
                self.next_blink = global_tick + 80 + (global_tick % 200);
            }
        }
    }

    /// Returns (is_blinking, pulse[0..1], emotion) for the renderer.
    pub fn render_state(&self, tick: u64) -> (bool, f32, Emotion) {
        let blink_duration = 6u64;
        let is_blinking = self.emotion != Emotion::Thinking
            && tick >= self.blink_tick
            && tick < self.blink_tick + blink_duration;

        // Pulse frequency varies by emotion
        let freq = match self.emotion {
            Emotion::Idle => 0.02,
            Emotion::Thinking => 0.08,
            Emotion::Speaking => 0.12,
            Emotion::Alert => 0.18,
            Emotion::Happy => 0.05,
        };
        let pulse = (tick as f32 * freq).sin() * 0.5 + 0.5;

        (is_blinking, pulse, self.emotion)
    }

    pub fn emotion(&self) -> Emotion {
        self.emotion
    }
}
