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
    vx: f32,
    vy: f32,
    next_wander: u64,
    rng: SmallRng,
    placed: bool,
    speech_text: Option<String>,
    anchor: Option<(f32, f32)>,
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
            next_blink: 120,
            emotion_since: 0,
            pos_x: 0.0,
            pos_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            vx: 0.0,
            vy: 0.0,
            next_wander: 50,
            rng: SmallRng::seed_from_u64(0xA11A7E),
            placed: false,
            speech_text: None,
            anchor: None,
        }
    }

    pub fn set_speech(&mut self, text: Option<&str>) {
        self.speech_text = text.map(|s| s.to_string());
    }

    pub fn speech(&self) -> Option<&str> {
        self.speech_text.as_deref()
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.pos_x = x;
        self.pos_y = y;
        self.target_x = x;
        self.target_y = y;
        self.placed = true;
    }

    pub fn set_target_position(&mut self, x: f32, y: f32) {
        self.anchor = Some((x, y));
        self.target_x = x;
        self.target_y = y;
    }

    pub fn clear_anchor(&mut self) {
        self.anchor = None;
    }

    pub fn set_emotion(&mut self, emotion: Emotion) {
        if self.emotion != emotion {
            self.emotion = emotion;
            self.emotion_since = self.tick;
        }
    }

    /// Advances blink/pulse state and physics. The character smoothly drifts
    /// between observation waypoints with continuous organic floating wave physics.
    pub fn tick(&mut self, global_tick: u64, width: u32, height: u32) {
        self.tick = global_tick;

        // Blink logic (suppress blinking while thinking)
        if self.emotion != Emotion::Thinking && global_tick >= self.next_blink {
            self.blink_tick = global_tick;
            // Next blink: 80–240 frames (~1.3–4.0s @ 60fps)
            self.next_blink = global_tick + 80 + (global_tick % 160);
        }

        // Wander bounds across the desktop canvas
        let margin_x = 140.0f32;
        let min_x = margin_x;
        let max_x = (width as f32 - margin_x).max(min_x + 10.0);
        let min_y = 140.0f32;
        let max_y = (height as f32 - 180.0).max(min_y + 10.0);

        if !self.placed {
            self.pos_x = (width as f32) * 0.5;
            self.pos_y = (height as f32) * 0.42;
            self.target_x = self.pos_x;
            self.target_y = self.pos_y;
            self.placed = true;
        }

        // Pick dynamic roaming waypoints every 140..260 ticks (~2.3-4.3s)
        if global_tick >= self.next_wander {
            if let Some((ax, ay)) = self.anchor {
                // Orbit gently around anchor focal point
                let ox = self.rng.gen_range(-48.0..=48.0);
                let oy = self.rng.gen_range(-32.0..=32.0);
                self.target_x = (ax + ox).clamp(min_x, max_x);
                self.target_y = (ay + oy).clamp(min_y, max_y);
            } else {
                // Free roaming across wide desktop canvas
                self.target_x = self.rng.gen_range(min_x..=max_x);
                self.target_y = self.rng.gen_range(min_y..=max_y);
            }
            self.next_wander = global_tick + self.rng.gen_range(140..260);
        }

        // Spring physics (Emil Kowalski / Apple-style spring model with velocity momentum)
        let stiffness = match self.emotion {
            Emotion::Thinking => 0.032,
            Emotion::Alert => 0.060,
            _ => 0.038,
        };
        let damping = 0.82f32;

        let ax = (self.target_x - self.pos_x) * stiffness;
        let ay = (self.target_y - self.pos_y) * stiffness;

        self.vx = (self.vx + ax) * damping;
        self.vy = (self.vy + ay) * damping;

        self.pos_x += self.vx;
        self.pos_y += self.vy;
    }

    /// Current on-screen center of the character, in pixels.
    /// Incorporates continuous harmonic micro-floating wave physics so the
    /// orb is always subtly alive, breathing, and moving.
    pub fn position(&self) -> (f32, f32) {
        let t = self.tick as f32;
        let wave_x = (t * 0.024).sin() * 9.0 + (t * 0.052).cos() * 4.5;
        let wave_y = (t * 0.033).cos() * 8.0 + (t * 0.068).sin() * 4.0;
        (self.pos_x + wave_x, self.pos_y + wave_y)
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
