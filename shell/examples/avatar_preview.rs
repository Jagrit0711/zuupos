//! Offline avatar/HUD preview — renders the same drawing code the live
//! compositor uses into PNG files, with no window or display server needed.
//!
//! Run with: `cargo run -p zuupos-shell --example avatar_preview`
//! Output: target/avatar_preview/*.png

use std::fs;
use std::path::Path;
use zuupos_shell::avatar::{Avatar, Emotion};
use zuupos_shell::cards::{Card, CardDetailRow, CardType};
use zuupos_shell::ipc::{PlanStep, StepStatus, SystemMetrics};
use zuupos_shell::renderer::Renderer;
use zuupos_shell::theme::ThemeMode;

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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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

    // ── 1. Clean Desktop Boot Preview (Clean Canvas, Roaming Avatar, Interactive Dock)
    {
        let mut clean_renderer = Renderer::new_headless(fw, fh);
        clean_renderer.theme_mode = ThemeMode::Light;
        let mut clean_avatar = Avatar::new();
        clean_avatar.set_position(640.0, 380.0);
        clean_avatar.set_speech(Some(
            "Hey! I'm Zuupos, your AI operating system.\nType anything in the dock below to get started!",
        ));
        let mut tick = 0u64;
        for _ in 0..40 {
            tick += 1;
            clean_avatar.tick(tick, fw, fh);
            clean_renderer.render(tick, &[], &clean_avatar, false);
        }
        save_png(&clean_renderer, &out_dir.join("clean_desktop_idle.png"));
    }

    // ── 2. Live Interactive Keyboard Typing State Preview ────────────────────
    {
        let mut typing_renderer = Renderer::new_headless(fw, fh);
        typing_renderer.theme_mode = ThemeMode::Light;
        typing_renderer.prompt_input = "book hotel in Kyoto".to_string();
        let mut typing_avatar = Avatar::new();
        typing_avatar.set_position(640.0, 380.0);
        typing_avatar.set_emotion(Emotion::Thinking);
        typing_avatar.set_speech(Some(
            "Listening...\nPress Enter or click the send button\nto dispatch.",
        ));
        let mut tick = 0u64;
        for _ in 0..35 {
            tick += 1;
            typing_avatar.tick(tick, fw, fh);
            typing_renderer.render(tick, &[], &typing_avatar, false);
        }
        save_png(&typing_renderer, &out_dir.join("clean_desktop_typing.png"));
    }

    // ── 3. Taby Ethereal Light Mode Hotel Booking & Visual References Preview ───
    let mut booking_card = Card::new_booking(
        "Aman Kyoto",
        "1 Ogocho, Kita Ward",
        "BOOKING FINALIZED",
        vec![
            CardDetailRow::new("calendar", "Nov 12 - Nov 16", "4 Nights"),
            CardDetailRow::new("bed", "Susuki Pavilion", "2 Guests"),
        ],
    );
    booking_card.pos_x = Some(150);
    booking_card.pos_y = Some(180);
    booking_card.custom_width = Some(340);
    booking_card.custom_height = Some(265);

    let mut gallery_card = Card::new_gallery(
        "Visual References Collected",
        vec![
            "assets/cards/kyoto_street.jpg".to_string(),
            "assets/cards/mount_fuji.jpg".to_string(),
            "assets/cards/japan_night.jpg".to_string(),
            "assets/cards/modern_pavilion.jpg".to_string(),
            "assets/cards/luxury_bedroom.jpg".to_string(),
        ],
    );
    gallery_card.pos_x = Some(710);
    gallery_card.pos_y = Some(140);
    gallery_card.custom_width = Some(440);
    gallery_card.custom_height = Some(440);

    let light_cards = vec![&booking_card, &gallery_card];

    let mut light_renderer = Renderer::new_headless(fw, fh);
    light_renderer.theme_mode = ThemeMode::Light;

    let mut light_avatar = Avatar::new();
    light_avatar.set_position(590.0, 500.0);
    light_avatar.set_speech(Some(
        "I've finalized your booking\nand gathered the visual\nreferences you wanted.\nWhere to next?",
    ));

    let mut light_tick = 0u64;
    for _ in 0..30 {
        light_tick += 1;
        light_avatar.tick(light_tick, fw, fh);
        light_renderer.render(light_tick, &light_cards, &light_avatar, false);
    }
    save_png(&light_renderer, &out_dir.join("taby_light_hotel_booking.png"));
    save_png(&light_renderer, &out_dir.join("hotel_booking_spawned.png"));

    // ── 4. System Telemetry & Diagnostics Intent Spawned ─────────────────────
    {
        let card_sys = Card {
            id: 1,
            title: "Kernel Diagnostics".to_string(),
            body: "PMM: 1989 MiB usable. Heap: 32 MiB. VMM 4 GiB identity mapped. Ring 3 syscalls active.".to_string(),
            card_type: CardType::Metric,
            max_age: 600,
            app_id: Some("kernel".to_string()),
            kind: Some("monitor".to_string()),
            pos_x: Some(180),
            pos_y: Some(180),
            custom_width: Some(360),
            custom_height: Some(220),
            detail_rows: vec![
                CardDetailRow::new("cpu", "CPU Usage", "12.4%"),
                CardDetailRow::new("mem", "Memory Allocated", "9.1 / 16.0 GB"),
            ],
            ..Default::default()
        };

        let card_sched = Card {
            id: 2,
            title: "Task Scheduler".to_string(),
            body: "PID 1 (init/hello) running. IPC zero-copy ring buffer latency <1.2 us.".to_string(),
            card_type: CardType::Info,
            max_age: 600,
            app_id: Some("sched".to_string()),
            pos_x: Some(580),
            pos_y: Some(180),
            custom_width: Some(360),
            custom_height: Some(220),
            ..Default::default()
        };

        let sys_cards = vec![&card_sys, &card_sched];
        let mut sys_renderer = Renderer::new_headless(fw, fh);
        sys_renderer.theme_mode = ThemeMode::Light;
        let mut sys_avatar = Avatar::new();
        sys_avatar.set_position(480.0, 480.0);
        sys_avatar.set_emotion(Emotion::Speaking);
        sys_avatar.set_speech(Some(
            "System telemetry online.\nKernel running smooth at 60 FPS.\nAll diagnostics nominal.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            sys_avatar.tick(tick, fw, fh);
            sys_renderer.render(tick, &sys_cards, &sys_avatar, false);
        }
        save_png(&sys_renderer, &out_dir.join("system_diagnostics_spawned.png"));
    }

    // ── 5. Interactive Stopwatch App Card ────────────────────────────────────
    {
        let mut stopwatch_card = Card::new_stopwatch();
        stopwatch_card.age_ticks = 2480; // ~41.33 seconds
        let sw_cards = vec![&stopwatch_card];
        let mut sw_renderer = Renderer::new_headless(fw, fh);
        sw_renderer.theme_mode = ThemeMode::Light;
        let mut sw_avatar = Avatar::new();
        sw_avatar.set_position(740.0, 480.0);
        sw_avatar.set_emotion(Emotion::Happy);
        sw_avatar.set_speech(Some(
            "Stopwatch launched!\nFocus timer is active and running.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            sw_avatar.tick(tick, fw, fh);
            sw_renderer.render(tick, &sw_cards, &sw_avatar, false);
        }
        save_png(&sw_renderer, &out_dir.join("stopwatch_app_spawned.png"));
    }

    // ── 6. Scratchpad Notes App Card ─────────────────────────────────────────
    {
        let notes_card = Card::new_notes(
            "System Scratchpad",
            vec![
                "Zero-copy ring buffer IPC latency <1.2µs",
                "60 FPS locked compositor frame pacing",
                "Bare-metal FAT32 filesystem driver",
            ],
        );
        let notes_cards = vec![&notes_card];
        let mut notes_renderer = Renderer::new_headless(fw, fh);
        notes_renderer.theme_mode = ThemeMode::Light;
        let mut notes_avatar = Avatar::new();
        notes_avatar.set_position(740.0, 480.0);
        notes_avatar.set_emotion(Emotion::Happy);
        notes_avatar.set_speech(Some(
            "Opened Scratchpad notes.\nTasks are synced with local memory.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            notes_avatar.tick(tick, fw, fh);
            notes_renderer.render(tick, &notes_cards, &notes_avatar, false);
        }
        save_png(&notes_renderer, &out_dir.join("notes_app_spawned.png"));
    }

    // ── 7. System Calculator App Card ────────────────────────────────────────
    {
        let calc_card = Card::new_calculator(
            "256 * 1024 / 8 = 32,768 KB",
            "32,768 KB (32 MB)",
            "0x8000",
        );
        let calc_cards = vec![&calc_card];
        let mut calc_renderer = Renderer::new_headless(fw, fh);
        calc_renderer.theme_mode = ThemeMode::Light;
        let mut calc_avatar = Avatar::new();
        calc_avatar.set_position(720.0, 480.0);
        calc_avatar.set_emotion(Emotion::Happy);
        calc_avatar.set_speech(Some(
            "Calculator online.\nResult and conversions computed.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            calc_avatar.tick(tick, fw, fh);
            calc_renderer.render(tick, &calc_cards, &calc_avatar, false);
        }
        save_png(&calc_renderer, &out_dir.join("calculator_app_spawned.png"));
    }

    // ── 8. Cyberpunk Lo-Fi Music Player Card ─────────────────────────────────
    {
        let music_card = Card::new_music("Cyberpunk Lo-Fi (Radio)", "Zuupos Synth Studio");
        let music_cards = vec![&music_card];
        let mut music_renderer = Renderer::new_headless(fw, fh);
        music_renderer.theme_mode = ThemeMode::Light;
        let mut music_avatar = Avatar::new();
        music_avatar.set_position(740.0, 480.0);
        music_avatar.set_emotion(Emotion::Happy);
        music_avatar.set_speech(Some(
            "Now playing Cyberpunk Lo-Fi.\nAmbient audio visualizer active.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            music_avatar.tick(tick, fw, fh);
            music_renderer.render(tick, &music_cards, &music_avatar, false);
        }
        save_png(&music_renderer, &out_dir.join("music_app_spawned.png"));
    }

    // ── 9. Real Host Hardware Telemetry Card ─────────────────────────────────
    {
        let mut sys = sysinfo::System::new_all();
        let sys_card = Card::new_real_system(&mut sys);
        let sys_cards = vec![&sys_card];
        let mut sys_renderer = Renderer::new_headless(fw, fh);
        sys_renderer.theme_mode = ThemeMode::Light;
        let mut sys_avatar = Avatar::new();
        sys_avatar.set_position(740.0, 480.0);
        sys_avatar.set_emotion(Emotion::Speaking);
        sys_avatar.set_speech(Some(
            "Host hardware telemetry online.\nReal CPU load, memory, & processes.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            sys_avatar.tick(tick, fw, fh);
            sys_renderer.render(tick, &sys_cards, &sys_avatar, false);
        }
        save_png(&sys_renderer, &out_dir.join("real_telemetry_spawned.png"));
    }

    // ── 10. Real Workspace File Explorer Card ────────────────────────────────
    {
        let files_card = Card::new_file_browser(".");
        let files_cards = vec![&files_card];
        let mut files_renderer = Renderer::new_headless(fw, fh);
        files_renderer.theme_mode = ThemeMode::Light;
        let mut files_avatar = Avatar::new();
        files_avatar.set_position(740.0, 480.0);
        files_avatar.set_emotion(Emotion::Happy);
        files_avatar.set_speech(Some(
            "Workspace explorer online.\nListed actual local filesystem contents.",
        ));
        let mut tick = 0u64;
        for _ in 0..30 {
            tick += 1;
            files_avatar.tick(tick, fw, fh);
            files_renderer.render(tick, &files_cards, &files_avatar, false);
        }
        save_png(&files_renderer, &out_dir.join("file_explorer_spawned.png"));
    }

    // ── 11. Real Dynamic Math Evaluator Card ─────────────────────────────────
    {
        if let Some((expr, dec, hex)) = Card::evaluate_math("450 * 12 + 250") {
            let math_card = Card::new_calculator(&expr, &dec, &hex);
            let math_cards = vec![&math_card];
            let mut math_renderer = Renderer::new_headless(fw, fh);
            math_renderer.theme_mode = ThemeMode::Light;
            let mut math_avatar = Avatar::new();
            math_avatar.set_position(720.0, 480.0);
            math_avatar.set_emotion(Emotion::Happy);
            math_avatar.set_speech(Some(
                "Calculated 450 * 12 + 250 = 5,650.\nHex: 0x1612.",
            ));
            let mut tick = 0u64;
            for _ in 0..30 {
                tick += 1;
                math_avatar.tick(tick, fw, fh);
                math_renderer.render(tick, &math_cards, &math_avatar, false);
            }
            save_png(&math_renderer, &out_dir.join("real_math_eval_spawned.png"));
        }
    }
}


