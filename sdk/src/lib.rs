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
    pub use crate::app::{App, AppError, AppManifest, AppState, ProcessInfo, ProcessState};
    pub use crate::capabilities::{Capability, CapabilitySet, CapabilityType};
    pub use crate::tools::{
        FunctionTool, ParameterProperty, ToolCall, ToolHandler, ToolParameters, ToolRegistry,
        ToolResult, ToolSchema,
    };
    pub use crate::ui::{
        ActionType, Card, CardAction, CardHeight, CardPriority, CardStyle, CardType, CardWidth,
        Emotion, HudMetrics, ShellMessage, Widget,
    };
}
