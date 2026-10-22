//! ZuuposOS Agentic Shell — main entry point
//!
//! Runs the winit event loop and the IPC listener for agentd commands.
//! On bare metal this becomes a framebuffer-direct process; on macOS host
//! it runs as a native window for rapid development.

mod avatar;
mod cards;
mod compositor;
mod ipc;
mod renderer;
mod theme;

use compositor::run_shell;
use log::info;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("╔════════════════════════════════════════╗");
    info!("║      ZuuposOS Shell v0.1 starting      ║");
    info!("║    AI-driven agentic UI compositor     ║");
    info!("╚════════════════════════════════════════╝");

    run_shell();
}
