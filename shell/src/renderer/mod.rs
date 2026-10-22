//! Software renderer — CPU pixel pipeline.
//!
//! Renders directly to a softbuffer framebuffer:
//!   Layer 0: Animated background (noise + gradient)
//!   Layer 1: Status bar (top, 32px)
//!   Layer 2: Floating cards (right side, AI-decided)
//!   Layer 3: Avatar face (bottom-left)
//!   Layer 4: Boot splash overlay (fade out)

use crate::avatar::Avatar;
use crate::cards::{Card, CardType};
use crate::theme::*;
use fontdue::{Font, FontSettings};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    surface: Surface<Arc<Window>, Arc<Window>>,
    width: u32,
    height: u32,
    buf: Vec<u32>,
    font: Font,
    font_bold: Font,
    noise: Vec<u8>, // pre-baked noise texture
}

impl Renderer {
    pub fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let ctx = Context::new(window.clone()).expect("softbuffer context failed");
        let surface = Surface::new(&ctx, window).expect("softbuffer surface failed");

        // Load system fonts (embedded as bytes for portability)
        let font_bytes = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
        let bold_bytes = include_bytes!("../../../assets/fonts/Inter-Bold.ttf");

        let font = Font::from_bytes(
            font_bytes.as_ref(),
            FontSettings::default(),
        )
        .expect("Failed to load Inter Regular");

        let font_bold = Font::from_bytes(
            bold_bytes.as_ref(),
            FontSettings::default(),
        )
        .expect("Failed to load Inter Bold");

        // Pre-bake a noise texture for background variation
        let noise: Vec<u8> = (0..512 * 512)
            .map(|i: usize| {
                let x = i % 512;
                let y = i / 512;
                // Cheap deterministic hash noise
                let h = (x.wrapping_mul(1619) ^ y.wrapping_mul(31337)).wrapping_mul(0x45d9f3b);
                (h >> 24) as u8
            })
            .collect();

        let buf = vec![0u32; (width * height) as usize];

        Self {
            surface,
            width,
            height,
            buf,
            font,
            font_bold,
            noise,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buf.resize((width * height) as usize, 0);
    }

    pub fn render(&mut self, tick: u64, cards: &[&Card], avatar: &Avatar, in_boot: bool) {
        self.draw_background(tick);
        self.draw_statusbar(tick);
        self.draw_cards(cards, tick);
        self.draw_avatar(avatar, tick);

        if in_boot {
            self.draw_boot_overlay(tick);
        }

        self.present();
    }

    // ── Background ──────────────────────────────────────────────────────────

    fn draw_background(&mut self, tick: u64) {
        let w = self.width;
        let h = self.height;
        let t = tick as f32 * 0.008;

        // Two animated radial gradient centers
        let cx1 = (0.3 + (t * 0.7).sin() * 0.15) * w as f32;
        let cy1 = (0.4 + (t * 0.5).cos() * 0.1) * h as f32;
        let cx2 = (0.7 + (t * 0.4).cos() * 0.12) * w as f32;
        let cy2 = (0.6 + (t * 0.6).sin() * 0.1) * h as f32;

        for y in 0..h {
            for x in 0..w {
                let dx1 = (x as f32 - cx1) / w as f32;
                let dy1 = (y as f32 - cy1) / h as f32;
                let d1 = (dx1 * dx1 + dy1 * dy1).sqrt().min(1.0);

                let dx2 = (x as f32 - cx2) / w as f32;
                let dy2 = (y as f32 - cy2) / h as f32;
                let d2 = (dx2 * dx2 + dy2 * dy2).sqrt().min(1.0);

                // Purple haze from center 1
                let p1 = ((1.0 - d1 * 2.5).max(0.0)).powf(2.5);
                // Blue wash from center 2
                let p2 = ((1.0 - d2 * 2.8).max(0.0)).powf(2.5);

                // Noise grain
                let nx = (x % 512) as usize;
                let ny = (y % 512) as usize;
                let noise_val = self.noise[ny * 512 + nx] as f32 / 255.0 * 0.018;

                let r = (0.02 + p1 * 0.18 + p2 * 0.04 + noise_val).min(1.0);
                let g = (0.01 + p1 * 0.03 + p2 * 0.06 + noise_val * 0.5).min(1.0);
                let b = (0.05 + p1 * 0.22 + p2 * 0.28 + noise_val).min(1.0);

                let ir = (r * 255.0) as u32;
                let ig = (g * 255.0) as u32;
                let ib = (b * 255.0) as u32;
                self.buf[(y * w + x) as usize] = (ir << 16) | (ig << 8) | ib;
            }
        }
    }

    // ── Status Bar ───────────────────────────────────────────────────────────

    fn draw_statusbar(&mut self, tick: u64) {
        let w = self.width;
        // Fill bar
        for x in 0..w {
            for y in 0..STATUSBAR_HEIGHT {
                let alpha = if y == STATUSBAR_HEIGHT - 1 { 0x1a1a2e } else { STATUSBAR_BG };
                self.buf[(y * w + x) as usize] = alpha;
            }
        }

        // "ZUUPOS OS" brand text
        self.draw_text("ZUUPOS OS", 16, 8, 11.0, ACCENT_PURPLE, false);

        // Time (right side)
        let now = chrono::Local::now();
        let time_str = now.format("%H:%M").to_string();
        let tx = w as i32 - 60;
        self.draw_text(&time_str, tx, 8, 11.0, TEXT_SECONDARY, false);

        // Agent status indicator dot (pulsing)
        let dot_x = w / 2;
        let pulse = (tick as f32 * 0.05).sin() * 0.5 + 0.5;
        let dot_brightness = 0.4 + pulse * 0.6;
        let dot_color = lerp_color(0x1a3a1a, ACCENT_GREEN, dot_brightness);
        self.fill_circle(dot_x, STATUSBAR_HEIGHT / 2, 4, dot_color);

        // Status text next to dot
        self.draw_text("Agent Active", (dot_x + 10) as i32, 8, 10.0, TEXT_SECONDARY, false);
    }

    // ── Cards ─────────────────────────────────────────────────────────────────

    fn draw_cards(&mut self, cards: &[&Card], tick: u64) {
        let card_w: u32 = 340;
        let card_h: u32 = 110;
        let margin: u32 = 16;
        let start_x = self.width.saturating_sub(card_w + margin);
        let start_y = STATUSBAR_HEIGHT + margin;

        for (i, card) in cards.iter().enumerate() {
            let y = start_y + i as u32 * (card_h + 10);
            if y + card_h > self.height { break; }

            // Slide-in animation: cards enter from right
            let age = card.age_ticks;
            let slide_progress = (age as f32 / 20.0).min(1.0);
            let slide_offset = ((1.0 - ease_out_cubic(slide_progress)) * 60.0) as u32;
            let x = start_x + slide_offset;

            // Fade out near end of life
            let fade = if card.age_ticks > card.max_age.saturating_sub(60) {
                1.0 - (card.age_ticks - card.max_age.saturating_sub(60)) as f32 / 60.0
            } else {
                1.0
            };

            let (bg, accent) = match card.card_type {
                CardType::Info => (CARD_BG_INFO, ACCENT_PURPLE),
                CardType::Task => (CARD_BG_TASK, ACCENT_BLUE),
                CardType::Alert => (CARD_BG_ALERT, ACCENT_RED),
                CardType::Weather => (CARD_BG_WEATHER, ACCENT_CYAN),
            };

            // Card body
            self.fill_rect(x, y, card_w, card_h, bg);

            // Accent left border (3px)
            self.fill_rect(x, y, 3, card_h, accent);

            // Top highlight line
            self.fill_rect(x + 3, y, card_w - 3, 1, blend(bg, 0xffffff, 0.08));

            // Bottom shadow line
            self.fill_rect(x, y + card_h - 1, card_w, 1, 0x000000);

            // Card type icon dot (top-right)
            self.fill_circle(x + card_w - 14, y + 14, 5, accent);

            // Title text
            self.draw_text(&card.title, (x + 14) as i32, (y + 14) as i32, 12.0, TEXT_PRIMARY, true);

            // Body text (word-wrapped, 2 lines max)
            let body_lines = wrap_text(&card.body, 36);
            for (li, line) in body_lines.iter().enumerate().take(3) {
                self.draw_text(
                    line,
                    (x + 14) as i32,
                    (y + 34 + li as u32 * 18) as i32,
                    10.0,
                    TEXT_SECONDARY,
                    false,
                );
            }

            // Age progress bar at bottom
            let progress = 1.0 - (card.age_ticks as f32 / card.max_age as f32);
            let bar_w = ((card_w - 3) as f32 * progress) as u32;
            self.fill_rect(x + 3, y + card_h - 3, bar_w, 2, blend(accent, 0x000000, 0.3));
        }
    }

    // ── Avatar ───────────────────────────────────────────────────────────────

    fn draw_avatar(&mut self, avatar: &Avatar, tick: u64) {
        let size: u32 = 90;
        let margin: u32 = 20;
        let x = margin;
        let y = self.height.saturating_sub(size + margin + STATUSBAR_HEIGHT);

        let (blink, pulse, emotion) = avatar.render_state(tick);

        // Outer glow ring (pulsing)
        let glow_r = size / 2 + 10;
        let glow_intensity = pulse * 0.3;
        let glow_color = lerp_color(BG_BASE, ACCENT_PURPLE, glow_intensity);
        self.fill_circle(x + size / 2, y + size / 2, glow_r, glow_color);

        // Second inner glow
        let inner_glow = lerp_color(BG_BASE, AVATAR_GLOW, pulse * 0.5);
        self.fill_circle(x + size / 2, y + size / 2, size / 2 + 4, inner_glow);

        // Avatar background circle
        self.fill_circle(x + size / 2, y + size / 2, size / 2, AVATAR_BG);

        // Face circle (slightly lighter)
        self.fill_circle(x + size / 2, y + size / 2, size / 2 - 4, AVATAR_FACE);

        // Eyes
        let eye_y_base = y + size / 2 - 10;
        let left_eye_x = x + size / 2 - 14;
        let right_eye_x = x + size / 2 + 14;

        let eye_h = if blink { 1u32 } else { 7u32 };
        let eye_w = 9u32;

        // Emotion-based eye color
        let eye_color = match emotion {
            crate::avatar::Emotion::Idle => AVATAR_EYE,
            crate::avatar::Emotion::Thinking => ACCENT_AMBER,
            crate::avatar::Emotion::Speaking => ACCENT_GREEN,
            crate::avatar::Emotion::Alert => ACCENT_RED,
            crate::avatar::Emotion::Happy => 0x34d399,
        };

        // Left eye
        if !blink {
            self.fill_circle(left_eye_x, eye_y_base, eye_w / 2, eye_color);
        } else {
            self.fill_rect(left_eye_x - eye_w / 2, eye_y_base, eye_w, 2, eye_color);
        }
        // Right eye
        if !blink {
            self.fill_circle(right_eye_x, eye_y_base, eye_w / 2, eye_color);
        } else {
            self.fill_rect(right_eye_x - eye_w / 2, eye_y_base, eye_w, 2, eye_color);
        }

        // Thinking dots animation (when emotion = Thinking)
        if emotion == crate::avatar::Emotion::Thinking {
            for i in 0..3u32 {
                let dot_phase = ((tick as f32 * 0.15 + i as f32 * 0.8).sin() * 0.5 + 0.5) as f32;
                let dot_color = lerp_color(AVATAR_FACE, ACCENT_AMBER, dot_phase);
                let dx = x + size / 2 - 8 + i * 8;
                let dy = y + size / 2 + 12;
                self.fill_circle(dx, dy, 3, dot_color);
            }
        } else {
            // Normal smile
            let smile_y = y + size / 2 + 12;
            let smile_x = x + size / 2;
            let smile_w = 20u32;
            // Simple arc approximated as rectangle with rounded ends
            self.fill_rect(smile_x - smile_w / 2, smile_y, smile_w, 3, eye_color);
            self.fill_circle(smile_x - smile_w / 2, smile_y + 1, 2, eye_color);
            self.fill_circle(smile_x + smile_w / 2, smile_y + 1, 2, eye_color);
        }

        // Avatar label
        self.draw_text("ZuuposOS", (x) as i32, (y + size + 4) as i32, 9.0, TEXT_DIM, false);
    }

    // ── Boot Overlay ─────────────────────────────────────────────────────────

    fn draw_boot_overlay(&mut self, tick: u64) {
        let alpha = 1.0 - (tick as f32 / 120.0).min(1.0);
        if alpha <= 0.0 { return; }

        // Dim overlay
        let dim = (alpha * 220.0) as u32;
        for px in self.buf.iter_mut() {
            let r = ((*px >> 16) & 0xff).min(dim >> 2);
            let g = ((*px >> 8) & 0xff).min(dim >> 2);
            let b = (*px & 0xff).min(dim >> 2);
            *px = (r << 16) | (g << 8) | b;
        }

        // Center logo text
        let cx = (self.width / 2) as i32 - 70;
        let cy = (self.height / 2) as i32 - 20;
        let logo_alpha = (alpha * 255.0) as u8;
        let logo_color = blend(BG_BASE, ACCENT_PURPLE, alpha);
        self.draw_text("ZUUPOS OS", cx, cy, 32.0, logo_color, true);

        let sub_color = blend(BG_BASE, TEXT_SECONDARY, alpha * 0.8);
        self.draw_text("AI-Native Agentic Operating System", cx - 30, cy + 44, 13.0, sub_color, false);

        // Loading dots
        let dots_done = (tick / 30) as usize;
        let dot_str: String = (0..4).map(|i| if i < dots_done % 5 { '●' } else { '○' }).collect();
        let dots_color = blend(BG_BASE, ACCENT_PURPLE, alpha * 0.6);
        self.draw_text(&dot_str, cx + 50, cy + 70, 14.0, dots_color, false);
    }

    // ── Primitive Drawing ─────────────────────────────────────────────────────

    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        if x >= self.width || y >= self.height { return; }
        for py in y..y2 {
            for px in x..x2 {
                self.buf[(py * self.width + px) as usize] = color & 0x00ff_ffff;
            }
        }
    }

    fn fill_circle(&mut self, cx: u32, cy: u32, r: u32, color: u32) {
        if r == 0 { return; }
        let x0 = cx.saturating_sub(r);
        let x1 = (cx + r).min(self.width.saturating_sub(1));
        let y0 = cy.saturating_sub(r);
        let y1 = (cy + r).min(self.height.saturating_sub(1));
        let r2 = (r * r) as i64;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let dx = px as i64 - cx as i64;
                let dy = py as i64 - cy as i64;
                if dx * dx + dy * dy <= r2 {
                    self.buf[(py * self.width + px) as usize] = color & 0x00ff_ffff;
                }
            }
        }
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, size: f32, color: u32, bold: bool) {
        let font = if bold { &self.font_bold } else { &self.font };
        let r = ((color >> 16) & 0xff) as f32 / 255.0;
        let g = ((color >> 8) & 0xff) as f32 / 255.0;
        let b = (color & 0xff) as f32 / 255.0;

        let mut cursor_x = x;
        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, size);
            let bx = cursor_x + metrics.xmin;
            let by = y + (size as i32) - metrics.ymin - metrics.height as i32;
            for (i, &cov) in bitmap.iter().enumerate() {
                if cov == 0 { continue; }
                let px = bx + (i % metrics.width) as i32;
                let py = by + (i / metrics.width) as i32;
                if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
                    continue;
                }
                let alpha = cov as f32 / 255.0;
                let idx = (py as u32 * self.width + px as u32) as usize;
                let bg = self.buf[idx];
                let br = ((bg >> 16) & 0xff) as f32 / 255.0;
                let bg_g = ((bg >> 8) & 0xff) as f32 / 255.0;
                let bb = (bg & 0xff) as f32 / 255.0;
                let fr = (r * alpha + br * (1.0 - alpha)).min(1.0);
                let fg = (g * alpha + bg_g * (1.0 - alpha)).min(1.0);
                let fb = (b * alpha + bb * (1.0 - alpha)).min(1.0);
                self.buf[idx] = ((fr * 255.0) as u32) << 16
                    | ((fg * 255.0) as u32) << 8
                    | (fb * 255.0) as u32;
            }
            cursor_x += metrics.advance_width as i32;
        }
    }

    fn present(&mut self) {
        let w = NonZeroU32::new(self.width).unwrap();
        let h = NonZeroU32::new(self.height).unwrap();
        self.surface.resize(w, h).unwrap();
        let mut sbuf = self.surface.buffer_mut().unwrap();
        sbuf.copy_from_slice(&self.buf);
        sbuf.present().unwrap();
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ar = ((a >> 16) & 0xff) as f32;
    let ag = ((a >> 8) & 0xff) as f32;
    let ab = (a & 0xff) as f32;
    let br = ((b >> 16) & 0xff) as f32;
    let bg_ = ((b >> 8) & 0xff) as f32;
    let bb = (b & 0xff) as f32;
    let r = (ar + (br - ar) * t) as u32;
    let g = (ag + (bg_ - ag) * t) as u32;
    let b_ = (ab + (bb - ab) * t) as u32;
    (r << 16) | (g << 8) | b_
}

fn blend(bg: u32, fg: u32, alpha: f32) -> u32 {
    lerp_color(bg, fg, alpha)
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_chars {
            if !current.is_empty() {
                lines.push(current.trim().to_string());
                current = String::new();
            }
        }
        if !current.is_empty() { current.push(' '); }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current.trim().to_string());
    }
    lines
}
