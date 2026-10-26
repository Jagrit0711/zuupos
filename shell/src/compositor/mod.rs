//! ZuuposOS Compositor — winit 0.30 ApplicationHandler implementation.
//!
//! This is the root of the shell's rendering pipeline. It owns the window,
//! surface, renderer, card manager, and avatar. It receives IPC events from
//! agentd and translates them into visual state changes.

use crate::avatar::Avatar;
use crate::cards::{Card, CardHitZone, CardManager, CardType};
use crate::ipc::{IpcReceiver, ShellEvent};
use crate::renderer::Renderer;
use crate::theme::ThemeMode;
use log::info;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

#[derive(PartialEq, Clone, Copy)]
pub enum BootPhase {
    Splash,  // First 120 ticks — boot animation
    Running, // Normal agent loop
}

struct ShellApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    card_manager: CardManager,
    avatar: Avatar,
    ipc_rx: Arc<Mutex<IpcReceiver>>,
    tick: u64,
    boot_phase: BootPhase,
    width: u32,
    height: u32,
    cursor_pos: (f64, f64),
    prompt_input: String,
    llm: crate::llm::LlmManager,
    is_generating: bool,
    active_send_ticks: u32,
    last_frame: std::time::Instant,
}

impl ShellApp {
    fn new() -> Self {
        let card_manager = CardManager::new();

        let mut avatar = Avatar::new();
        avatar.set_position(640.0, 380.0);
        avatar.set_speech(Some(
            "Hey! I'm Zuupos, your AI operating system.\nType anything in the dock below to get started!",
        ));

        let ipc_rx = Arc::new(Mutex::new(IpcReceiver::new()));
        let ipc_clone = ipc_rx.clone();
        std::thread::spawn(move || {
            ipc_clone.lock().unwrap().listen();
        });

        let llm = crate::llm::LlmManager::new();

        Self {
            window: None,
            renderer: None,
            card_manager,
            avatar,
            ipc_rx,
            tick: 0,
            boot_phase: BootPhase::Running,
            width: 1280,
            height: 800,
            cursor_pos: (0.0, 0.0),
            prompt_input: String::new(),
            llm,
            is_generating: false,
            active_send_ticks: 0,
            last_frame: std::time::Instant::now(),
        }
    }

    fn handle_prompt_submission(&mut self, prompt: String) {
        let trimmed = prompt.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        let lower = trimmed.to_lowercase();
        info!("[compositor] User prompt: '{}'", trimmed);

        // Tactile press state for send button
        self.active_send_ticks = 8;
        self.avatar.set_emotion(crate::avatar::Emotion::Thinking);

        if lower.contains("book")
            || lower.contains("hotel")
            || lower.contains("kyoto")
            || lower.contains("travel")
        {
            self.card_manager.clear();

            let mut booking_card = Card::new_booking(
                "Aman Kyoto",
                "1 Ogocho, Kita Ward",
                "BOOKING FINALIZED",
                vec![
                    crate::cards::CardDetailRow::new("calendar", "Nov 12 - Nov 16", "4 Nights"),
                    crate::cards::CardDetailRow::new("bed", "Susuki Pavilion", "2 Guests"),
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

            self.card_manager.add_card(booking_card);
            self.card_manager.add_card(gallery_card);

            self.avatar.set_target_position(590.0, 500.0);
            self.avatar.set_emotion(crate::avatar::Emotion::Speaking);
            self.avatar.set_speech(Some(
                "I've finalized your booking\nand gathered the visual\nreferences you wanted.\nWhere to next?",
            ));
        } else if lower.contains("sys")
            || lower.contains("stat")
            || lower.contains("cpu")
            || lower.contains("mem")
            || lower.contains("diag")
        {
            self.card_manager.clear();

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
                    crate::cards::CardDetailRow::new("cpu", "CPU Usage", "12.4%"),
                    crate::cards::CardDetailRow::new("mem", "Memory Allocated", "9.1 / 16.0 GB"),
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

            self.card_manager.add_card(card_sys);
            self.card_manager.add_card(card_sched);

            self.avatar.set_target_position(480.0, 480.0);
            self.avatar.set_emotion(crate::avatar::Emotion::Speaking);
            self.avatar.set_speech(Some(
                "System telemetry online.\nKernel running smooth at 60 FPS.\nAll diagnostics nominal.",
            ));
        } else if lower.contains("research")
            || lower.contains("search")
            || lower.contains("web")
            || lower.contains("paper")
        {
            self.card_manager.clear();

            let card_research = Card {
                id: 1,
                title: "Autonomous Web Research".to_string(),
                body: "Analyzed microkernel IPC vs capability rights. Discovered zero-copy ring buffer reduces context switch overhead by 34%.".to_string(),
                card_type: CardType::Info,
                max_age: 600,
                app_id: Some("research".to_string()),
                pos_x: Some(360),
                pos_y: Some(170),
                custom_width: Some(440),
                custom_height: Some(240),
                actions: vec![
                    crate::cards::CardAction::new("Inspect Notes", "inspect"),
                    crate::cards::CardAction::new("Save to Memory", "save"),
                ],
                ..Default::default()
            };

            self.card_manager.add_card(card_research);
            self.avatar.set_target_position(580.0, 480.0);
            self.avatar.set_emotion(crate::avatar::Emotion::Speaking);
            self.avatar.set_speech(Some(
                "Synthesized 4 research papers.\nWork-stealing scheduler architecture\nready for inspection.",
            ));
        } else if lower.contains("help") || lower.contains("command") || lower.contains("what can") {
            self.card_manager.clear();
            let card_help = Card {
                id: 1,
                title: "Zuupos AI Agent Capabilities".to_string(),
                body: "Type any prompt in the dock below to direct your autonomous agent:".to_string(),
                card_type: CardType::Info,
                max_age: 600,
                app_id: Some("help".to_string()),
                pos_x: Some(340),
                pos_y: Some(180),
                custom_width: Some(480),
                custom_height: Some(250),
                detail_rows: vec![
                    crate::cards::CardDetailRow::new("calendar", "Travel Booking", "'book hotel in kyoto'"),
                    crate::cards::CardDetailRow::new("cpu", "Kernel Diagnostics", "'system stats' or 'cpu'"),
                    crate::cards::CardDetailRow::new("pin", "Research Agent", "'research papers'"),
                    crate::cards::CardDetailRow::new("bed", "Visual Themes", "'theme' (toggle light/dark)"),
                ],
                actions: vec![
                    crate::cards::CardAction::new("Clear Canvas", "clear"),
                ],
                ..Default::default()
            };
            self.card_manager.add_card(card_help);
            self.avatar.set_target_position(580.0, 480.0);
            self.avatar.set_emotion(crate::avatar::Emotion::Speaking);
            self.avatar.set_speech(Some(
                "Here are actions you can take!\nType in the dock or press Enter\nto dispatch autonomous agents.",
            ));
        } else if lower.contains("clear") || lower.contains("clean") || lower.contains("reset") {
            self.card_manager.clear();
            self.avatar.clear_anchor();
            self.avatar.set_emotion(crate::avatar::Emotion::Happy);
            self.avatar.set_speech(Some(
                "Canvas cleared!\nWhat would you like to explore next?",
            ));
        } else if lower.contains("theme") || lower.contains("dark") || lower.contains("light") {
            if let Some(renderer) = &mut self.renderer {
                renderer.theme_mode = match renderer.theme_mode {
                    ThemeMode::Light => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                };
            }
            self.avatar.set_emotion(crate::avatar::Emotion::Happy);
            self.avatar.set_speech(Some(
                "Switched color theme palette.\nHow does this feel?",
            ));
        } else {
            // Live asynchronous Ollama LLM execution!
            self.avatar.set_speech(Some("Thinking… generating response with local AI."));
            self.avatar.set_emotion(crate::avatar::Emotion::Thinking);
            self.is_generating = true;
            self.llm.submit(&trimmed);
        }
    }
}

impl ApplicationHandler for ShellApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("ZuuposOS — Agentic Shell")
            .with_inner_size(LogicalSize::new(1280.0, 800.0))
            .with_decorations(true);

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        let size = window.inner_size();
        self.width = size.width.max(1);
        self.height = size.height.max(1);

        let renderer = Renderer::new(window.clone(), self.width, self.height);

        self.window = Some(window.clone());
        self.renderer = Some(renderer);

        info!(
            "[compositor] Shell started ({}x{})",
            self.width, self.height
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.clone() else { return };
        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("[compositor] Window close requested");
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let (mx, my) = (self.cursor_pos.0 as u32, self.cursor_pos.1 as u32);

                // Check Dock interactive Send button
                let dock_w = 640u32;
                let _dock_h = 50u32;
                let dock_x = (self.width.saturating_sub(dock_w)) / 2;
                let dock_y = self.height.saturating_sub(70);
                if !self.prompt_input.is_empty() {
                    let send_w = 46u32;
                    let send_x = dock_x + dock_w - send_w - 8;
                    let send_y = dock_y + 9;
                    if mx >= send_x && mx <= send_x + send_w && my >= send_y && my <= send_y + 32 {
                        let prompt = std::mem::take(&mut self.prompt_input);
                        self.handle_prompt_submission(prompt);
                        window.request_redraw();
                        return;
                    }
                }

                if let Some((card_id, zone)) = self.card_manager.hit_test(mx, my, self.width) {
                    match zone {
                        CardHitZone::Dismiss => {
                            info!("[compositor] Mouse click: Dismiss card {}", card_id);
                            self.card_manager.dismiss_card(card_id);
                        }
                        CardHitZone::MinimizeToggle => {
                            info!("[compositor] Mouse click: Toggle minimize card {}", card_id);
                            self.card_manager.toggle_minimized(card_id);
                        }
                        CardHitZone::CardBody => {
                            info!("[compositor] Mouse click: Focus card {}", card_id);
                            self.card_manager.focus_card(card_id);
                        }
                        CardHitZone::Action(idx) => {
                            info!("[compositor] Mouse click: Action {} on card {}", idx, card_id);
                            self.card_manager.focus_card(card_id);
                        }
                    }
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        text,
                        ..
                    },
                ..
            } => {
                match logical_key {
                    Key::Named(NamedKey::Enter) => {
                        if !self.prompt_input.trim().is_empty() {
                            let prompt = std::mem::take(&mut self.prompt_input);
                            self.handle_prompt_submission(prompt);
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        self.prompt_input.pop();
                    }
                    Key::Named(NamedKey::Escape) => {
                        if !self.prompt_input.is_empty() {
                            self.prompt_input.clear();
                        } else if let Some(fid) = self.card_manager.focused_card().map(|c| c.id) {
                            self.card_manager.dismiss_card(fid);
                        } else {
                            info!("[compositor] Escape pressed, exiting shell");
                            event_loop.exit();
                        }
                    }
                    Key::Named(NamedKey::Tab) | Key::Named(NamedKey::ArrowDown) => {
                        self.card_manager.focus_next();
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        self.card_manager.focus_prev();
                    }
                    _ => {
                        // Capture printable Unicode text
                        if let Some(ref txt) = text {
                            for ch in txt.chars() {
                                if !ch.is_control() {
                                    self.prompt_input.push(ch);
                                    self.avatar.set_emotion(crate::avatar::Emotion::Thinking);
                                }
                            }
                        } else if let Key::Character(ref s) = logical_key {
                            for ch in s.chars() {
                                if !ch.is_control() {
                                    self.prompt_input.push(ch);
                                    self.avatar.set_emotion(crate::avatar::Emotion::Thinking);
                                }
                            }
                        }
                    }
                }
                // Emil Kowalski principle: Never animate keyboard actions.
                // Request immediate redraw for instantaneous 0ms keystroke feedback!
                window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width.max(1), new_size.height.max(1));
                }
                self.width = new_size.width.max(1);
                self.height = new_size.height.max(1);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.tick += 1;

                if self.active_send_ticks > 0 {
                    self.active_send_ticks -= 1;
                }

                // Poll asynchronous LLM response
                if let Some(response) = self.llm.poll_response() {
                    self.is_generating = false;
                    self.avatar.set_emotion(crate::avatar::Emotion::Speaking);

                    let speech = if response.text.len() > 140 {
                        format!("{}…", &response.text[..137])
                    } else {
                        response.text.clone()
                    };
                    self.avatar.set_speech(Some(&speech));

                    // Display complete LLM answer in an AI response card
                    let card = Card {
                        id: 0,
                        title: format!("AI: {}", response.model),
                        body: response.text,
                        card_type: CardType::Info,
                        max_age: 1800,
                        app_id: Some("ollama".to_string()),
                        pos_x: Some(300),
                        pos_y: Some(170),
                        custom_width: Some(520),
                        custom_height: Some(260),
                        ..Default::default()
                    };
                    self.card_manager.add_card(card);
                }

                // Boot phase transition
                if self.boot_phase == BootPhase::Splash && self.tick > 120 {
                    self.boot_phase = BootPhase::Running;
                    self.card_manager.add_card(Card {
                        title: "Agent Online".to_string(),
                        body: "ZuuposOS is thinking. LLM warming up…".to_string(),
                        card_type: CardType::Info,
                        max_age: 400,
                        ..Default::default()
                    });
                }

                // Poll IPC for events from agentd
                if let Ok(mut rx) = self.ipc_rx.try_lock() {
                    for evt in rx.drain_events() {
                        match evt {
                            ShellEvent::Card(card) => {
                                self.card_manager.add_card(card);
                            }
                            ShellEvent::AvatarEmotion(emotion) => {
                                self.avatar.set_emotion(emotion);
                            }
                            ShellEvent::StatusUpdate(text) => {
                                log::debug!("[ipc] Status: {}", text);
                            }
                            ShellEvent::Metrics(metrics) => {
                                if let Some(renderer) = &mut self.renderer {
                                    renderer.metrics = Some(metrics);
                                }
                            }
                            ShellEvent::TaskPlan(steps) => {
                                if let Some(renderer) = &mut self.renderer {
                                    renderer.plan_steps = steps;
                                }
                            }
                            ShellEvent::DockUpdate(_entry) => {
                                log::debug!("[ipc] Dock update received");
                            }
                            ShellEvent::Notification {
                                title,
                                body,
                                urgency: _,
                            } => {
                                self.card_manager.add_card(Card {
                                    title,
                                    body,
                                    card_type: CardType::Alert,
                                    max_age: 350,
                                    ..Default::default()
                                });
                            }
                            ShellEvent::BootStarted => {
                                self.boot_phase = BootPhase::Splash;
                            }
                            ShellEvent::AgentReady => {
                                self.boot_phase = BootPhase::Running;
                            }
                        }
                    }
                }

                self.avatar.tick(self.tick, self.width, self.height);
                self.card_manager.tick();

                let in_boot = self.boot_phase == BootPhase::Splash;
                if let Some(renderer) = &mut self.renderer {
                    renderer.prompt_input = self.prompt_input.clone();
                    renderer.llm_model = self.llm.model_name().to_string();
                    renderer.is_generating = self.is_generating;
                    renderer.active_send_ticks = self.active_send_ticks;
                    renderer.render(
                        self.tick,
                        &self.card_manager.active_cards(),
                        &self.avatar,
                        in_boot,
                    );
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Emil Kowalski & 60 FPS frame pacing: locked 16.6ms intervals
        // Prevents CPU busy-spinning, cooling down hardware while maintaining silky smooth 60 FPS
        let frame_duration = std::time::Duration::from_micros(16_667);
        let elapsed = self.last_frame.elapsed();

        if elapsed >= frame_duration {
            self.last_frame = std::time::Instant::now();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        } else {
            let next_frame = self.last_frame + frame_duration;
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame));
        }
    }
}

pub fn run_shell() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = ShellApp::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
