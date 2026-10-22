//! Taby-style AI avatar face with blink and pulse animations.

pub struct Avatar {
    tick: u64,
    blink_tick: u64,
    next_blink: u64,
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            tick: 0,
            blink_tick: 0,
            next_blink: 120,
        }
    }

    pub fn tick(&mut self, global_tick: u64) {
        self.tick = global_tick;
        if global_tick >= self.next_blink {
            self.blink_tick = global_tick;
            // Next blink in 100–300 frames (randomish)
            self.next_blink = global_tick + 100 + (global_tick % 200);
        }
    }

    /// Returns (is_blinking, pulse_phase[0.0..1.0])
    pub fn state(&self) -> (bool, f32) {
        let blink_duration = 8u64;
        let is_blinking = self.tick >= self.blink_tick
            && self.tick < self.blink_tick + blink_duration;
        let pulse = (self.tick as f32 * 0.03).sin() * 0.5 + 0.5;
        (is_blinking, pulse)
    }
}
