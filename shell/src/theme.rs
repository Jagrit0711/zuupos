//! Theme constants — ZuuposOS warm dark palette.
//!
//! Deliberately *not* the purple/blue "AI gradient" look: one confident warm
//! accent (amber/coral) plus a single cool counterpoint (teal), on a neutral
//! warm-charcoal base, closer to a native dark-mode OS than a sci-fi void.
#![allow(dead_code)]

// Background layers
pub const BG_BASE: u32 = 0x121113; // Warm near-black charcoal
pub const BG_GRADIENT_A: u32 = 0x1a1510; // Warm dark umber
pub const BG_GRADIENT_B: u32 = 0x15120f; // Warm dark graphite

// Accent colors — one warm primary, one cool counterpoint, semantic states
pub const ACCENT_PRIMARY: u32 = 0xff9142; // Brand accent — warm amber/coral
pub const ACCENT_BLUE: u32 = 0x5aa9e6; // Muted info blue (secondary use only)
pub const ACCENT_GREEN: u32 = 0x3ddc84; // Success / active
pub const ACCENT_AMBER: u32 = 0xf5b84e; // Thinking (close kin to primary, slightly more yellow)
pub const ACCENT_RED: u32 = 0xef6f6f; // Alert — softer coral-red, not a harsh siren red
pub const ACCENT_CYAN: u32 = 0x45c4b0; // Processing — the one cool counterpoint accent

// Text
pub const TEXT_PRIMARY: u32 = 0xf1ece4;
pub const TEXT_SECONDARY: u32 = 0xa89d8d;
pub const TEXT_DIM: u32 = 0x6b6258;

// Card backgrounds — warm neutral, each with a faint lean toward its accent
pub const CARD_BG_INFO: u32 = 0x18140f;
pub const CARD_BG_TASK: u32 = 0x141210;
pub const CARD_BG_ALERT: u32 = 0x1a1210;
pub const CARD_BG_WEATHER: u32 = 0x101614;
pub const CARD_BG_METRIC: u32 = 0x181410;
pub const CARD_BG_PLAN: u32 = 0x10160f;

// Borders / hairlines
pub const BORDER: u32 = 0x332c22;
pub const PANEL_BG: u32 = 0x15110c;

// Avatar — warm ember glow instead of violet
pub const AVATAR_BG: u32 = 0x201a14;
pub const AVATAR_FACE: u32 = 0x2b2219;
pub const AVATAR_EYE: u32 = 0xffcf8a; // Idle eye tint — soft warm gold
pub const AVATAR_GLOW: u32 = 0x7a3f12;

// Status bar
pub const STATUSBAR_BG: u32 = 0x100e0b;
pub const STATUSBAR_HEIGHT: u32 = 30;
