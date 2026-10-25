//! Theme constants — ZuuposOS dual-palette design system.
//!
//! Supports both:
//! 1. Light Mode (Taby Ethereal): Serene pale mint/lavender ambient mesh,
//!    floating crisp white cards with soft drop shadows, lilac-periwinkle
//!    gradient pearl avatar, frosted speech bubbles, and centered pill dock.
//! 2. Dark Mode (Warm Charcoal): Warm near-black base with amber/coral accents.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

// ─── Light Mode (Taby Ethereal) ─────────────────────────────────────────────

// Ambient gradient background
pub const LIGHT_BG_BASE: u32 = 0xf6f9f8;        // Ethereal mist white
pub const LIGHT_BG_MINT: u32 = 0xe2f1ea;        // Soft mint/sage ambient glow (top-left)
pub const LIGHT_BG_LAVENDER: u32 = 0xeee7f7;    // Delicate lilac/periwinkle blush (center)
pub const LIGHT_BG_CREAM: u32 = 0xfcfdfd;       // Luminous highlight

// Floating cards
pub const LIGHT_CARD_BG: u32 = 0xffffff;        // Pure white floating card
pub const LIGHT_CARD_BORDER: u32 = 0xe7ecf2;    // 1px subtle boundary
pub const LIGHT_CARD_SHADOW: u32 = 0x121b28;    // Soft diffuse drop shadow tint

// Typography
pub const LIGHT_TEXT_TITLE: u32 = 0x181a1f;     // "Aman Kyoto" / "Visual References Collected"
pub const LIGHT_TEXT_BODY: u32 = 0x242831;      // Primary card text
pub const LIGHT_TEXT_MUTED: u32 = 0x717886;     // "1 Ogocho, Kita Ward", "4 Nights", "2 Guests"
pub const LIGHT_TEXT_DIM: u32 = 0x9ba3b2;

// Semantic badges
pub const STATUS_FINALIZED_TEXT: u32 = 0x059669;// Emerald green "✓ BOOKING FINALIZED"
pub const STATUS_FINALIZED_BG: u32 = 0xecfdf5;  // Emerald pill background
pub const STATUS_FINALIZED_BORDER: u32 = 0xa7f3d0;

// Avatar — Soft floating lilac/periwinkle gradient pearl orb
pub const AVATAR_LILAC_CORE: u32 = 0xd8c8fd;    // Dreamy lilac inner highlight
pub const AVATAR_PERIWINKLE: u32 = 0xb5c9fe;    // Soft periwinkle gradient body
pub const AVATAR_ROSE_GLOW: u32 = 0xf0d6fb;     // Gentle blush rim
pub const AVATAR_SHADOW: u32 = 0xc5bed4;

// Frosted Speech Bubble
pub const SPEECH_LIGHT_BG: u32 = 0xffffff;
pub const SPEECH_LIGHT_BORDER: u32 = 0xe2e6ee;
pub const SPEECH_LIGHT_TEXT_MUTED: u32 = 0x5a6372;
pub const SPEECH_LIGHT_TEXT_BOLD: u32 = 0x1a1d24;

// Floating Pill Dock (Bottom center)
pub const DOCK_LIGHT_BG: u32 = 0xffffff;
pub const DOCK_LIGHT_BORDER: u32 = 0xe4e8f0;
pub const DOCK_LIGHT_TEXT: u32 = 0x1f232b;
pub const DOCK_LIGHT_ICON: u32 = 0x667080;
pub const DOCK_MIC_BG: u32 = 0xebf0ff;          // Soft periwinkle circular button
pub const DOCK_MIC_ICON: u32 = 0x5064f5;        // Vibrant periwinkle blue mic glyph

// ─── Dark Mode (Warm Charcoal) ──────────────────────────────────────────────

pub const BG_BASE: u32 = 0x121113;              // Warm near-black charcoal
pub const BG_GRADIENT_A: u32 = 0x1a1510;        // Warm dark umber
pub const BG_GRADIENT_B: u32 = 0x15120f;        // Warm dark graphite

pub const ACCENT_PRIMARY: u32 = 0xff9142;       // Brand accent — warm amber/coral
pub const ACCENT_BLUE: u32 = 0x5aa9e6;          // Muted info blue
pub const ACCENT_GREEN: u32 = 0x3ddc84;         // Success / active
pub const ACCENT_AMBER: u32 = 0xf5b84e;         // Thinking
pub const ACCENT_RED: u32 = 0xef6f6f;           // Alert
pub const ACCENT_CYAN: u32 = 0x45c4b0;          // Processing

pub const TEXT_PRIMARY: u32 = 0xf1ece4;
pub const TEXT_SECONDARY: u32 = 0xa89d8d;
pub const TEXT_DIM: u32 = 0x6b6258;

pub const CARD_BG_INFO: u32 = 0x18140f;
pub const CARD_BG_TASK: u32 = 0x141210;
pub const CARD_BG_ALERT: u32 = 0x1a1210;
pub const CARD_BG_WEATHER: u32 = 0x101614;
pub const CARD_BG_METRIC: u32 = 0x181410;
pub const CARD_BG_PLAN: u32 = 0x10160f;

pub const BORDER: u32 = 0x332c22;
pub const PANEL_BG: u32 = 0x15110c;

pub const AVATAR_BG: u32 = 0x201a14;
pub const AVATAR_FACE: u32 = 0x2b2219;
pub const AVATAR_EYE: u32 = 0xffcf8a;
pub const AVATAR_GLOW: u32 = 0x7a3f12;

pub const STATUSBAR_BG: u32 = 0x100e0b;
pub const STATUSBAR_HEIGHT: u32 = 30;
