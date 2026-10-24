//! Avatar — the "face" of ZuuposOS.
//!
//! It roams the lower portion of the screen rather than sitting fixed in a
//! corner — a character that's present in the space, not a static icon.
//! Its emotion state is driven by the agentd via IPC, making it visually
//! represent what the AI is doing.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Emotion {
    #[default]
    Idle,
    Thinking,
    Speaking,
    Alert,
    Happy,
    Processing,
}

pub struct Avatar {
    emotion: Emotion,
    tick: u64,
    blink_tick: u64,
    next_blink: u64,
    emotion_since: u64,
    pos_x: f32,
    pos_y: f32,
    target_x: f32,
    target_y: f32,
    next_wander: u64,
    rng: SmallRng,
    placed: bool,
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            emotion: Emotion::Idle,
            tick: 0,
            blink_tick: 0,
            next_blink: 180,
            emotion_since: 0,
            pos_x: 0.0,
            pos_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            next_wander: 200,
            rng: SmallRng::seed_from_u64(0xA11A7E),
            placed: false,
        }
    }

    pub fn set_emotion(&mut self, emotion: Emotion) {
        if self.emotion != emotion {
            self.emotion = emotion;
            self.emotion_since = self.tick;
        }
    }

    /// Advances blink/pulse state and, given the current viewport size,
    /// drifts the character toward a wander target. It only wanders while
    /// Idle/Happy — Thinking/Speaking/Alert/Processing hold still, which
    /// reads as the character actually concentrating rather than fidgeting.
    pub fn tick(&mut self, global_tick: u64, width: u32, height: u32) {
        self.tick = global_tick;

        // Blink logic (suppress blinking while thinking)
        if self.emotion != Emotion::Thinking && global_tick >= self.next_blink {
            self.blink_tick = global_tick;
            // Next blink: 80–280 frames (~1.3–4.6s @ 60fps)
            self.next_blink = global_tick + 80 + (global_tick % 200);
        }

        // Wander bounds: lower-left ~58% of the screen, clear of the top
        // menu bar and the card column docked on the right.
        let margin = 70.0f32;
        let min_x = margin;
        let max_x = (width as f32 * 0.58).max(min_x + 1.0);
        let min_y = (height as f32 * 0.42).max(60.0);
        let max_y = (height as f32 - 110.0).max(min_y + 1.0);

        if !self.placed {
            self.pos_x = min_x + (max_x - min_x) * 0.18;
            self.pos_y = max_y - 8.0;
            self.target_x = self.pos_x;
            self.target_y = self.pos_y;
            self.placed = true;
        }

        let wandering = matches!(self.emotion, Emotion::Idle | Emotion::Happy);
        if wandering && global_tick >= self.next_wander {
            self.target_x = self.rng.gen_range(min_x..=max_x);
            self.target_y = self.rng.gen_range(min_y..=max_y);
            self.next_wander = global_tick + self.rng.gen_range(260..520);
        }
        if wandering {
            self.pos_x += (self.target_x - self.pos_x) * 0.012;
            self.pos_y += (self.target_y - self.pos_y) * 0.012;
        }
    }

    /// Current on-screen center of the character, in pixels.
    pub fn position(&self) -> (f32, f32) {
        (self.pos_x, self.pos_y)
    }

    /// Horizontal travel direction, for the renderer to flip the face so it
    /// looks like it's actually walking rather than sliding sideways.
    pub fn facing_right(&self) -> bool {
        self.target_x >= self.pos_x
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
            Emotion::Processing => 0.10,
        };
        let pulse = (tick as f32 * freq).sin() * 0.5 + 0.5;

        (is_blinking, pulse, self.emotion)
    }

    #[allow(dead_code)]
    pub fn emotion(&self) -> Emotion {
        self.emotion
    }
}
