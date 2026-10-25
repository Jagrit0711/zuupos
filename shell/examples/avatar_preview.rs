//! Offline avatar/HUD preview — renders the same drawing code the live
//! compositor uses into PNG files, with no window or display server needed.
//!
//! Run with: `cargo run -p zuupos-shell --example avatar_preview`
//! Output: target/avatar_preview/*.png

use std::fs;
use std::path::Path;
use zuupos_shell::avatar::{Avatar, Emotion};
use zuupos_shell::cards::{Card, CardType};
use zuupos_shell::ipc::{PlanStep, StepStatus, SystemMetrics};
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

    let width = 640;
    let height = 320;

    for &(name, emotion) in EMOTIONS {
        let mut renderer = Renderer::new_headless(width, height);
        let mut avatar = Avatar::new();
        avatar.set_emotion(emotion);
        // Advance several ticks so blink/bounce/wave animations settle into
        // a representative mid-cycle frame instead of tick 0.
        let mut tick = 0u64;
        for _ in 0..40 {
            tick += 1;
            avatar.tick(tick, width, height);
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
        avatar.tick(tick, width, height);
        renderer.render(tick, &[], &avatar, false);
    }
    save_png(&renderer, &out_dir.join("avatar_idle_blink.png"));

    // Full-window frames at the default launch size, with realistic sample
    // content (a card, an active plan, live-looking metrics) so the whole
    // composed UI — menu bar, voice dock, cards, roaming character — can be
    // judged together instead of just the character in isolation.
    let (fw, fh) = (1280u32, 800u32);
    let card = Card {
        id: 1,
        title: "System Summary".to_string(),
        body: "CPU: 18%, Mem: 9.1/16GB, Load: 1.20".to_string(),
        card_type: CardType::Info,
        age_ticks: 10,
        max_age: 400,
        ..Default::default()
    };
    let plan = vec![
        PlanStep {
            index: 0,
            description: "Check system load".to_string(),
            status: StepStatus::Done,
        },
        PlanStep {
            index: 1,
            description: "Summarize for the user".to_string(),
            status: StepStatus::Active,
        },
        PlanStep {
            index: 2,
            description: "Dim display for evening".to_string(),
            status: StepStatus::Pending,
        },
    ];
    let metrics = SystemMetrics {
        cpu_usage: 18.0,
        mem_used_mb: 9318,
        mem_total_mb: 16384,
        battery_pct: None,
        load_avg_1m: 1.20,
    };

    for (label, emotion, warmup) in [
        ("full_ui_idle", Emotion::Idle, 30u64),
        ("full_ui_wandered", Emotion::Idle, 400u64),
        ("full_ui_speaking", Emotion::Speaking, 60u64),
    ] {
        let mut renderer = Renderer::new_headless(fw, fh);
        renderer.metrics = Some(metrics.clone());
        renderer.plan_steps = plan.clone();
        let mut avatar = Avatar::new();
        avatar.set_emotion(emotion);
        let mut tick = 0u64;
        for _ in 0..warmup {
            tick += 1;
            avatar.tick(tick, fw, fh);
            renderer.render(tick, &[&card], &avatar, false);
        }
        save_png(&renderer, &out_dir.join(format!("{label}.png")));
    }

    // ── Multi-Card & Multi-App Surface Preview ──────────────────────────────
    let card_research = Card {
        id: 1,
        title: "Microkernel IPC Findings".to_string(),
        body: "Compared seL4 capability transfers vs Zuupos channel rights. Zero-copy ring buffer delivers <1.2μs latency.".to_string(),
        card_type: CardType::Info,
        age_ticks: 25,
        max_age: 500,
        app_id: Some("research".to_string()),
        kind: Some("notes".to_string()),
        focused: true,
        minimized: false,
        actions: vec![
            zuupos_shell::cards::CardAction::new("Open Note", "open_note"),
            zuupos_shell::cards::CardAction::new("Dismiss", "dismiss"),
        ],
    };

    let card_sys = Card {
        id: 2,
        title: "Kernel Diagnostics".to_string(),
        body: "PMM: 1989 MiB usable. Heap: 32 MiB. VMM 4 GiB identity mapped.".to_string(),
        card_type: CardType::Metric,
        age_ticks: 60,
        max_age: 600,
        app_id: Some("kernel".to_string()),
        kind: Some("monitor".to_string()),
        focused: false,
        minimized: true,
        actions: vec![],
    };

    let card_task = Card {
        id: 3,
        title: "Autonomous Web Search".to_string(),
        body: "Discovered 4 papers on work-stealing schedulers in Rust bare-metal targets.".to_string(),
        card_type: CardType::Task,
        age_ticks: 15,
        max_age: 450,
        app_id: Some("agentd".to_string()),
        kind: Some("task".to_string()),
        focused: false,
        minimized: false,
        actions: vec![
            zuupos_shell::cards::CardAction::new("Inspect", "inspect"),
        ],
    };

    let multi_cards = vec![&card_research, &card_sys, &card_task];
    let mut renderer = Renderer::new_headless(fw, fh);
    renderer.metrics = Some(metrics.clone());
    renderer.plan_steps = plan;
    let mut avatar = Avatar::new();
    avatar.set_emotion(Emotion::Thinking);
    let mut tick = 0u64;
    for _ in 0..45 {
        tick += 1;
        avatar.tick(tick, fw, fh);
        renderer.render(tick, &multi_cards, &avatar, false);
    }
    save_png(&renderer, &out_dir.join("full_ui_multicard.png"));
}
