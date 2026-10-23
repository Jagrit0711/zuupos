//! ZuuposOS Compositor — winit 0.30 ApplicationHandler implementation.
//!
//! This is the root of the shell's rendering pipeline. It owns the window,
//! surface, renderer, card manager, and avatar. It receives IPC events from
//! agentd and translates them into visual state changes.

use crate::avatar::Avatar;
use crate::cards::{Card, CardManager, CardType};
use crate::ipc::{IpcReceiver, ShellEvent};
use crate::renderer::Renderer;
use log::info;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
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
}

impl ShellApp {
    fn new() -> Self {
        let mut card_manager = CardManager::new();
        card_manager.add_card(Card {
            title: "ZuuposOS".to_string(),
            body: "Agent runtime initializing…".to_string(),
            card_type: CardType::Info,
            max_age: 300,
            ..Default::default()
        });

        let avatar = Avatar::new();
        let ipc_rx = Arc::new(Mutex::new(IpcReceiver::new()));
        let ipc_clone = ipc_rx.clone();
        std::thread::spawn(move || {
            ipc_clone.lock().unwrap().listen();
        });

        Self {
            window: None,
            renderer: None,
            card_manager,
            avatar,
            ipc_rx,
            tick: 0,
            boot_phase: BootPhase::Splash,
            width: 1280,
            height: 800,
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
            event_loop.create_window(window_attrs).expect("Failed to create window"),
        );

        let size = window.inner_size();
        self.width = size.width.max(1);
        self.height = size.height.max(1);

        let renderer = Renderer::new(window.clone(), self.width, self.height);

        self.window = Some(window.clone());
        self.renderer = Some(renderer);

        info!("[compositor] Shell started ({}x{})", self.width, self.height);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else { return };
        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("[compositor] Window close requested");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                info!("[compositor] Escape pressed, exiting shell");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width.max(1), new_size.height.max(1));
                }
                self.width = new_size.width.max(1);
                self.height = new_size.height.max(1);
            }
            WindowEvent::RedrawRequested => {
                self.tick += 1;

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
                            ShellEvent::Notification { title, body, urgency: _ } => {
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

                self.avatar.tick(self.tick);
                self.card_manager.tick();

                let in_boot = self.boot_phase == BootPhase::Splash;
                if let Some(renderer) = &mut self.renderer {
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

pub fn run_shell() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ShellApp::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}