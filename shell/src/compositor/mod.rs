//! Framebuffer compositor — the root rendering engine.
//!
//! Manages a layered scene graph:
//!   Layer 0: Background (gradient / ambient animation)
//!   Layer 1: Active cards (AI-decided, slide in/out)
//!   Layer 2: Avatar face (always-on-top, Taby-style)
//!   Layer 3: System notifications
//!
//! Phase 1.7: Runs in a winit event loop for prototyping on the host OS.
//! Phase 2.0: Switches to direct framebuffer writes on bare metal.

use crate::avatar::Avatar;
use crate::cards::{Card, CardManager};
use crate::renderer::Renderer;
use log::info;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Compositor {
    pub width: u32,
    pub height: u32,
}

impl Compositor {
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        let window = WindowBuilder::new()
            .with_title("ZuuposOS — Agentic Shell")
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height))
            .with_decorations(true)
            .build(&event_loop)
            .expect("Failed to create window");

        let mut renderer = Renderer::new(&window, self.width, self.height);
        let mut card_manager = CardManager::new();
        let mut avatar = Avatar::new();
        let mut tick: u64 = 0;

        // Seed with a welcome card
        card_manager.add_card(Card {
            id: 0,
            title: "ZuuposOS Online".to_string(),
            body: "Agent loop active. Agentic shell running. Phase 1.7.".to_string(),
            card_type: crate::cards::CardType::Info,
            age_ticks: 0,
            max_age: 600,
        });

        info!("[compositor] Entering render loop...");

        event_loop
            .run(|event, elwt| {
                elwt.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        elwt.exit();
                    }
                    Event::AboutToWait => {
                        tick += 1;
                        card_manager.tick();
                        avatar.tick(tick);

                        renderer.begin_frame();
                        renderer.draw_background(tick);
                        renderer.draw_cards(&card_manager.active_cards());
                        renderer.draw_avatar(&avatar);
                        renderer.end_frame(&window);
                    }
                    _ => {}
                }
            })
            .expect("Event loop error");
    }
}
