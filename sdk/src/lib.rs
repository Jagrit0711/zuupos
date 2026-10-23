//! ZuuposOS Developer SDK
//!
//! Build AI-native apps for ZuuposOS.
//! Apps are capability-bound processes that register tools and receive
//! context from the agent runtime.

pub mod app;
pub mod capabilities;
pub mod tools;
pub mod ui;

/// Prelude for common imports
pub mod prelude {
    pub use crate::app::{App, AppManifest, AppState, AppError, ProcessInfo, ProcessState};
    pub use crate::capabilities::{Capability, CapabilitySet, CapabilityType};
    pub use crate::tools::{ToolSchema, ToolParameters, ParameterProperty, ToolCall, ToolResult, ToolHandler, ToolRegistry, FunctionTool};
    pub use crate::ui::{Card, CardType, CardPriority, CardAction, ActionType, CardStyle, CardWidth, CardHeight, Widget, Emotion, HudMetrics, ShellMessage};
}
