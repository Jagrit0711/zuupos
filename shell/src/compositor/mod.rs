//! ZuuposOS Compositor — winit 0.29 event loop implementation.
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
    dpi::LogicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

#[derive(PartialEq, Clone, Copy)]
pub enum BootPhase {
    Splash,  // First 120 ticks — boot animation
    Running, // Normal agent loop
}

pub fn run_shell() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("ZuuposOS — Agentic Shell")
            .with_inner_size(LogicalSize::new(1280.0, 800.0))
            .with_decorations(true)
            .with_transparent(false)
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let size = window.inner_size();
    let width = size.width.max(1);
    let height = size.height.max(1);
    let mut renderer = Renderer::new(window.clone(), width, height);

    let mut card_manager = CardManager::new();
    card_manager.add_card(Card {
        title: "ZuuposOS".to_string(),
        body: "Agent runtime initializing…".to_string(),
        card_type: CardType::Info,
        max_age: 300,
        ..Default::default()
    });

    let mut avatar = Avatar::new();
    let mut tick: u64 = 0;
    let mut boot_phase = BootPhase::Splash;

    let ipc_rx = Arc::new(Mutex::new(IpcReceiver::new()));
    let ipc_clone = ipc_rx.clone();
    std::thread::spawn(move || {
        ipc_clone.lock().unwrap().listen();
    });

    info!("[compositor] Shell started ({}x{})", width, height);

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            info!("[compositor] Window close requested");
                            elwt.exit();
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
                            elwt.exit();
                        }
                        WindowEvent::Resized(new_size) => {
                            renderer.resize(new_size.width.max(1), new_size.height.max(1));
                        }
                        WindowEvent::RedrawRequested => {
                            tick += 1;

                            // Boot phase transition
                            if boot_phase == BootPhase::Splash && tick > 120 {
                                boot_phase = BootPhase::Running;
                                card_manager.add_card(Card {
                                    title: "Agent Online".to_string(),
                                    body: "ZuuposOS is thinking. LLM warming up…".to_string(),
                                    card_type: CardType::Info,
                                    max_age: 400,
                                    ..Default::default()
                                });
                            }

                            // Poll IPC for events from agentd
                            if let Ok(mut rx) = ipc_rx.try_lock() {
                                for evt in rx.drain_events() {
                                    match evt {
                                        ShellEvent::Card(card) => {
                                            card_manager.add_card(card);
                                        }
                                        ShellEvent::AvatarEmotion(emotion) => {
                                            avatar.set_emotion(emotion);
                                        }
                                        ShellEvent::StatusUpdate(text) => {
                                            log::debug!("[ipc] Status: {}", text);
                                        }
                                    }
                                }
                            }

                            avatar.tick(tick);
                            card_manager.tick();

                            let in_boot = boot_phase == BootPhase::Splash;
                            renderer.render(
                                tick,
                                &card_manager.active_cards(),
                                &avatar,
                                in_boot,
                            );
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}
