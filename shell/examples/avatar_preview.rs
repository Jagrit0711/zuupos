//! Offline avatar/HUD preview — renders the same drawing code the live
//! compositor uses into PNG files, with no window or display server needed.
//!
//! Run with: `cargo run -p zuupos-shell --example avatar_preview`
//! Output: target/avatar_preview/*.png

use std::fs;
use std::path::Path;
use zuupos_shell::avatar::{Avatar, Emotion};
use zuupos_shell::renderer::Renderer;

const EMOTIONS: &[(&str, Emotion)] = &[
    ("idle", Emotion::Idle),
    ("thinking", Emotion::Thinking),
    ("speaking", Emotion::Speaking),
    ("alert", Emotion::Alert),
    ("happy", Emotion::Happy),
    ("processing", Emotion::Processing),
];

fn save_png(renderer: &Renderer, path: &Path) {
    let w = renderer.width();
    let h = renderer.height();
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for &px in renderer.pixels() {
        rgba.push(((px >> 16) & 0xff) as u8);
        rgba.push(((px >> 8) & 0xff) as u8);
        rgba.push((px & 0xff) as u8);
        rgba.push(0xff);
    }
    image::save_buffer(path, &rgba, w, h, image::ColorType::Rgba8)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    println!("wrote {}", path.display());
}

fn main() {
    let out_dir = Path::new("target/avatar_preview");
    fs::create_dir_all(out_dir).expect("create output dir");

    // Wide enough that the metrics HUD (bottom-right) doesn't overlap the
    // avatar (bottom-left), so the preview reads cleanly.
    let width = 420;
    let height = 300;

    for &(name, emotion) in EMOTIONS {
        let mut renderer = Renderer::new_headless(width, height);
        let mut avatar = Avatar::new();
        avatar.set_emotion(emotion);
        // Advance several ticks so blink/bounce/wave animations settle into
        // a representative mid-cycle frame instead of tick 0.
        let mut tick = 0u64;
        for _ in 0..40 {
            tick += 1;
            avatar.tick(tick);
            renderer.render(tick, &[], &avatar, false);
        }
        save_png(&renderer, &out_dir.join(format!("avatar_{name}.png")));
    }

    // Also capture a mid-blink frame for the idle state specifically.
    let mut renderer = Renderer::new_headless(width, height);
    let mut avatar = Avatar::new();
    avatar.set_emotion(Emotion::Idle);
    let mut tick = 0u64;
    // next_blink defaults to 180; land squarely inside the 6-tick blink window.
    for _ in 0..182 {
        tick += 1;
        avatar.tick(tick);
        renderer.render(tick, &[], &avatar, false);
    }
    save_png(&renderer, &out_dir.join("avatar_idle_blink.png"));
}
