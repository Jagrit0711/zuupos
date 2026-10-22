//! ZuuposOS Agentic Shell
//!
//! Phase 1.7: Framebuffer compositor + Taby-style avatar.
//! This process receives "card" commands from agentd over IPC
//! and renders them to the display. No traditional windows — only
//! AI-decided floating cards and the ambient avatar.

mod avatar;
mod cards;
mod compositor;
mod renderer;

use log::info;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("╔════════════════════════════════════════╗");
    info!("║      ZuuposOS Shell v0.1 starting      ║");
    info!("║    AI-driven agentic UI compositor     ║");
    info!("╚════════════════════════════════════════╝");

    let mut compositor = compositor::Compositor::new();
    compositor.run();
}
