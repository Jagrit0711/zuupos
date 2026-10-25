//! UI Card/Widget primitives for the agentic shell.
//!
//! The ZuuposOS shell renders dynamic cards based on agent decisions.
//! Apps can request cards to be displayed, updated, or dismissed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Info,
    Task,
    Alert,
    Weather,
    Metric,
    Media,
    Code,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub card_type: CardType,
    pub priority: CardPriority,
    pub title: String,
    pub body: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub actions: Vec<CardAction>,
    pub style: CardStyle,
    pub ttl_seconds: Option<u64>, // Time to live
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAction {
    pub label: String,
    pub action_type: ActionType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Dismiss,
    ToolCall,
    OpenUrl,
    OpenApp,
    SendIpc,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardStyle {
    pub background: Option<String>, // Hex color
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub icon: Option<String>,
    pub width: Option<CardWidth>,
    pub height: Option<CardHeight>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardWidth {
    Narrow,
    Medium,
    Wide,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardHeight {
    Auto,
    Compact,
    Standard,
    Expanded,
}

impl Card {
    pub fn new(card_type: CardType, title: &str, body: &str) -> Self {
        Self {
            id: 0, // Assigned by shell
            card_type,
            priority: CardPriority::Normal,
            title: title.to_string(),
            body: body.to_string(),
            metadata: HashMap::new(),
            actions: Vec::new(),
            style: CardStyle::default(),
            ttl_seconds: None,
            app_id: None,
            kind: None,
            focused: false,
            minimized: false,
        }
    }

    pub fn info(title: &str, body: &str) -> Self {
        Self::new(CardType::Info, title, body)
    }

    pub fn task(title: &str, body: &str) -> Self {
        Self::new(CardType::Task, title, body)
    }

    pub fn alert(title: &str, body: &str) -> Self {
        let mut card = Self::new(CardType::Alert, title, body);
        card.priority = CardPriority::High;
        card
    }

    pub fn metric(title: &str, body: &str) -> Self {
        Self::new(CardType::Metric, title, body)
    }

    pub fn with_priority(mut self, priority: CardPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_action(mut self, action: CardAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.ttl_seconds = Some(seconds);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    pub fn with_app_id(mut self, app_id: &str) -> Self {
        self.app_id = Some(app_id.to_string());
        self
    }

    pub fn with_kind(mut self, kind: &str) -> Self {
        self.kind = Some(kind.to_string());
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_minimized(mut self, minimized: bool) -> Self {
        self.minimized = minimized;
        self
    }
}

impl CardAction {
    pub fn dismiss(label: &str) -> Self {
        Self {
            label: label.to_string(),
            action_type: ActionType::Dismiss,
            payload: serde_json::Value::Null,
        }
    }

    pub fn tool_call(label: &str, tool: &str, args: serde_json::Value) -> Self {
        Self {
            label: label.to_string(),
            action_type: ActionType::ToolCall,
            payload: serde_json::json!({ "tool": tool, "args": args }),
        }
    }

    pub fn open_url(label: &str, url: &str) -> Self {
        Self {
            label: label.to_string(),
            action_type: ActionType::OpenUrl,
            payload: serde_json::json!({ "url": url }),
        }
    }

    pub fn open_app(label: &str, app: &str) -> Self {
        Self {
            label: label.to_string(),
            action_type: ActionType::OpenApp,
            payload: serde_json::json!({ "app": app }),
        }
    }

    pub fn send_ipc(label: &str, target: &str, message: serde_json::Value) -> Self {
        Self {
            label: label.to_string(),
            action_type: ActionType::SendIpc,
            payload: serde_json::json!({ "target": target, "message": message }),
        }
    }
}

/// Widget types for more complex UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Widget {
    Gauge(GaugeWidget),
    Chart(ChartWidget),
    List(ListWidget),
    Progress(ProgressWidget),
    TextInput(TextInputWidget),
    Button(ButtonWidget),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeWidget {
    pub label: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub color_zones: Vec<ColorZone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorZone {
    pub min: f64,
    pub max: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartWidget {
    pub label: String,
    pub data: Vec<ChartPoint>,
    pub chart_type: ChartType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWidget {
    pub title: String,
    pub items: Vec<ListItem>,
    pub selectable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub id: String,
    pub label: String,
    pub sublabel: Option<String>,
    pub icon: Option<String>,
    pub action: Option<CardAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressWidget {
    pub label: String,
    pub progress: f64, // 0.0 to 1.0
    pub indeterminate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputWidget {
    pub label: String,
    pub placeholder: String,
    pub value: String,
    pub on_submit: Option<CardAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonWidget {
    pub label: String,
    pub action: CardAction,
    pub style: ButtonStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

/// Avatar emotion states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    Idle,
    Happy,
    Thinking,
    Processing,
    Speaking,
    Sad,
    Surprised,
    Concerned,
}

impl Emotion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Emotion::Idle => "idle",
            Emotion::Happy => "happy",
            Emotion::Thinking => "thinking",
            Emotion::Processing => "processing",
            Emotion::Speaking => "speaking",
            Emotion::Sad => "sad",
            Emotion::Surprised => "surprised",
            Emotion::Concerned => "concerned",
        }
    }
}

/// HUD metrics for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudMetrics {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub memory_total_mb: u64,
    pub load_avg: f32,
    pub uptime_seconds: u64,
    pub network_rx_mbps: f32,
    pub network_tx_mbps: f32,
    pub temperature_c: Option<f32>,
}

/// IPC messages for shell communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellMessage {
    ShowCard(Card),
    UpdateCard { id: u64, updates: CardUpdates },
    DismissCard { id: u64 },
    SetAvatarEmotion(Emotion),
    SetStatusText(String),
    UpdateHud(HudMetrics),
    ShowWidget(Widget),
    UpdateWidget { id: u64, widget: Widget },
    DismissWidget { id: u64 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardUpdates {
    pub title: Option<String>,
    pub body: Option<String>,
    pub priority: Option<CardPriority>,
    pub actions: Option<Vec<CardAction>>,
    pub style: Option<CardStyle>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub app_id: Option<String>,
    pub kind: Option<String>,
    pub focused: Option<bool>,
    pub minimized: Option<bool>,
}
