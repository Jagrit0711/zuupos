//! ZuuposOS Agentic Shell — library surface.
//!
//! Exposed as a library (in addition to the `zuupos-shell` binary) so that
//! preview/testing tools — see `examples/avatar_preview.rs` — can render the
//! same drawing code used by the live compositor without needing a real
//! window or display server.

pub mod avatar;
pub mod cards;
pub mod compositor;
pub mod ipc;
pub mod renderer;
pub mod theme;
