//! Software renderer — CPU pixel pipeline for ZuuposOS shell.
//!
//! Layer stack (bottom → top):
//!   0. Animated background  — 3-center radial gradient + noise + drift
//!   1. Particle field        — floating ambient particles
//!   2. Status bar            — 32px top bar with time, brand, metrics
//!   3. Metrics HUD           — CPU/Mem gauges (bottom-right)
//!   4. Task plan panel       — active plan steps (left side, animated)
//!   5. Floating cards        — glassmorphism cards (right side)
//!   6. Avatar face           — animated AI face (bottom-left)
//!   7. Boot splash overlay   — fades out after boot

use crate::avatar::Avatar;
use crate::cards::{Card, CardType};
use crate::ipc::{PlanStep, StepStatus, SystemMetrics};
use crate::theme::*;
use fontdue::{Font, FontSettings};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::window::Window;

// ─── Particle ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32, // 0..1, fades out
    max_life: f32,
    size: f32,
    color: u32,
}

// ─── Renderer ────────────────────────────────────────────────────────────────

pub struct Renderer {
    /// `None` in headless mode (offline preview rendering — see
    /// `examples/avatar_preview.rs`), where there is no real window/surface
    /// to present frames to; every draw_* call still writes into `buf`.
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    width: u32,
    height: u32,
    buf: Vec<u32>,
    font: Font,
    font_bold: Font,
    noise: Vec<u8>,
    particles: Vec<Particle>,
    rng: SmallRng,
    /// Smoothed metrics for display (exponential moving average)
    smooth_cpu: f32,
    smooth_mem_pct: f32,
    /// Active plan steps from agent
    pub plan_steps: Vec<PlanStep>,
    /// Latest system metrics
    pub metrics: Option<SystemMetrics>,
    /// Waveform buffer for Speaking animation (32 samples)
    waveform: Vec<f32>,
}

impl Renderer {
    pub fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let ctx = Context::new(window.clone()).expect("softbuffer context failed");
        let surface = Surface::new(&ctx, window).expect("softbuffer surface failed");
        Self::new_internal(Some(surface), width, height)
    }

    /// Headless renderer with no backing window/surface — every draw_* call
    /// still writes into `buf`, `present()` is a no-op, and `pixels()` lets a
    /// caller (e.g. `examples/avatar_preview.rs`) export the frame to a file.
    pub fn new_headless(width: u32, height: u32) -> Self {
        Self::new_internal(None, width, height)
    }

    pub fn pixels(&self) -> &[u32] {
        &self.buf
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn new_internal(
        surface: Option<Surface<Arc<Window>, Arc<Window>>>,
        width: u32,
        height: u32,
    ) -> Self {
        // Embedded TrueType fonts
        let font_bytes = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
        let bold_bytes = include_bytes!("../../../assets/fonts/Inter-Bold.ttf");

        let font = Font::from_bytes(font_bytes.as_ref(), FontSettings::default())
            .expect("Inter Regular load failed");
        let font_bold = Font::from_bytes(bold_bytes.as_ref(), FontSettings::default())
            .expect("Inter Bold load failed");

        // Pre-bake deterministic noise texture (512×512)
        let noise: Vec<u8> = (0..512 * 512)
            .map(|i: usize| {
                let x = i % 512;
                let y = i / 512;
                let h = (x.wrapping_mul(1619) ^ y.wrapping_mul(31337)).wrapping_mul(0x45d9f3b);
                (h >> 24) as u8
            })
            .collect();

        let mut rng = SmallRng::seed_from_u64(0xdeadbeef);

        // Pre-seed particle field
        let particles: Vec<Particle> = (0..80)
            .map(|_| spawn_particle(&mut rng, width, height, true))
            .collect();

        // Initial waveform (flat)
        let waveform = vec![0.0f32; 32];

        Self {
            surface,
            width,
            height,
            buf: vec![0u32; (width * height) as usize],
            font,
            font_bold,
            noise,
            particles,
            rng,
            smooth_cpu: 0.0,
            smooth_mem_pct: 0.0,
            plan_steps: Vec::new(),
            metrics: None,
            waveform,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buf.resize((width * height) as usize, 0);
    }

    pub fn render(&mut self, tick: u64, cards: &[&Card], avatar: &Avatar, in_boot: bool) {
        // Update smooth metrics
        if let Some(m) = &self.metrics.clone() {
            let alpha = 0.08;
            self.smooth_cpu = self.smooth_cpu * (1.0 - alpha) + m.cpu_usage * alpha;
            let mem_pct = m.mem_used_mb as f32 / m.mem_total_mb.max(1) as f32;
            self.smooth_mem_pct = self.smooth_mem_pct * (1.0 - alpha) + mem_pct * alpha;
        }

        // Update waveform
        self.tick_waveform(avatar, tick);

        // Draw layers
        self.draw_background(tick);
        self.update_particles(tick);
        self.draw_particles();
        self.draw_statusbar(tick, avatar);
        self.draw_plan_panel(tick);
        self.draw_cards(cards, tick);
        self.draw_avatar(avatar, tick);
        self.draw_voice_dock(tick, avatar);

        if in_boot {
            self.draw_boot_overlay(tick);
        }

        self.present();
    }

    // ── Background ───────────────────────────────────────────────────────────

    fn draw_background(&mut self, tick: u64) {
        let w = self.width;
        let h = self.height;
        let t = tick as f32 * 0.006;

        // Three animated radial gradient centers
        let cx1 = (0.25 + (t * 0.7).sin() * 0.12) * w as f32;
        let cy1 = (0.35 + (t * 0.5).cos() * 0.10) * h as f32;

        let cx2 = (0.72 + (t * 0.4).cos() * 0.10) * w as f32;
        let cy2 = (0.65 + (t * 0.6).sin() * 0.10) * h as f32;

        let cx3 = (0.50 + (t * 0.3).sin() * 0.08) * w as f32;
        let cy3 = (0.20 + (t * 0.8).cos() * 0.08) * h as f32;

        for y in 0..h {
            for x in 0..w {
                let fx = x as f32;
                let fy = y as f32;

                let d1 = (((fx - cx1).powi(2) + (fy - cy1).powi(2)).sqrt() / w as f32).min(1.0);
                let d2 = (((fx - cx2).powi(2) + (fy - cy2).powi(2)).sqrt() / w as f32).min(1.0);
                let d3 = (((fx - cx3).powi(2) + (fy - cy3).powi(2)).sqrt() / w as f32).min(1.0);

                // One warm amber glow (the brand accent) plus a very faint
                // neutral counterpoint for depth — not three competing hues.
                // This replaces the old violet/indigo/blue "AI gradient" look.
                let p1 = (1.0 - d1 * 2.4).max(0.0).powf(2.2); // warm amber glow
                let p2 = (1.0 - d2 * 3.0).max(0.0).powf(2.8); // faint warm ember
                let p3 = (1.0 - d3 * 3.4).max(0.0).powf(3.2); // barely-there cool graphite, for quiet contrast

                // Noise grain
                let nx = (x % 512) as usize;
                let ny = (y % 512) as usize;
                let nv = self.noise[ny * 512 + nx] as f32 / 255.0 * 0.015;

                let r = (0.044 + p1 * 0.17 + p2 * 0.05 + p3 * 0.01 + nv).min(1.0);
                let g = (0.038 + p1 * 0.095 + p2 * 0.03 + p3 * 0.015 + nv * 0.8).min(1.0);
                let b = (0.034 + p1 * 0.02 + p2 * 0.015 + p3 * 0.022 + nv).min(1.0);

                let ir = (r * 255.0) as u32;
                let ig = (g * 255.0) as u32;
                let ib = (b * 255.0) as u32;
                self.buf[(y * w + x) as usize] = (ir << 16) | (ig << 8) | ib;
            }
        }
    }

    // ── Particle System ──────────────────────────────────────────────────────

    fn update_particles(&mut self, _tick: u64) {
        let w = self.width;
        let h = self.height;
        let dead: Vec<usize> = self
            .particles
            .iter()
            .enumerate()
            .filter(|(_, p)| p.life <= 0.0)
            .map(|(i, _)| i)
            .collect();

        for i in dead {
            self.particles[i] = spawn_particle(&mut self.rng, w, h, false);
        }

        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.life -= 1.0 / p.max_life;

            // Wrap horizontally
            if p.x < 0.0 {
                p.x += w as f32;
            }
            if p.x >= w as f32 {
                p.x -= w as f32;
            }
        }
    }

    fn draw_particles(&mut self) {
        let w = self.width;
        let h = self.height;
        for p in &self.particles {
            if p.life <= 0.0 {
                continue;
            }
            let alpha = (p.life * 0.55).min(1.0);
            let px = p.x as i32;
            let py = p.y as i32;
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                continue;
            }
            let idx = (py as u32 * w + px as u32) as usize;
            if idx < self.buf.len() {
                self.buf[idx] =
                    blend_pixel(self.buf[idx], p.color, alpha * (p.size / 3.0).min(1.0));
            }
            // Draw a small soft dot (radius 1)
            if p.size > 1.5 {
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = px + dx;
                        let ny = py + dy;
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let ni = (ny as u32 * w + nx as u32) as usize;
                        if ni < self.buf.len() {
                            let dist_a =
                                alpha * 0.35 * (1.0 - ((dx * dx + dy * dy) as f32).sqrt() * 0.5);
                            self.buf[ni] = blend_pixel(self.buf[ni], p.color, dist_a.max(0.0));
                        }
                    }
                }
            }
        }
    }

    // ── Status Bar ───────────────────────────────────────────────────────────

    /// Top menu bar — macOS-menu-bar-style: brand + live agent status on the
    /// left, system stats and the clock as compact text "extras" on the
    /// right. Replaces the old statusbar (which only showed a static
    /// "Agent Active" label and a CPU gauge) and the separate floating
    /// SYSTEM panel (which duplicated CPU/MEM here) — all system info now
    /// lives in exactly one place, matching where a native menu bar puts it.
    fn draw_statusbar(&mut self, tick: u64, avatar: &Avatar) {
        let w = self.width;
        for x in 0..w {
            for y in 0..STATUSBAR_HEIGHT {
                let base = if y == STATUSBAR_HEIGHT - 1 {
                    BORDER
                } else {
                    STATUSBAR_BG
                };
                self.buf[(y * w + x) as usize] =
                    blend_pixel(self.buf[(y * w + x) as usize], base, 0.92);
            }
        }

        // Brand
        self.fill_circle(14, STATUSBAR_HEIGHT / 2, 3, ACCENT_PRIMARY);
        self.draw_text("ZuuposOS", 24, 8, 11.0, TEXT_PRIMARY, true);
        self.draw_text("v0.5", 98, 9, 8.5, TEXT_DIM, false);
        self.fill_rect(134, 7, 1, STATUSBAR_HEIGHT - 14, BORDER);

        // Agent status — reflects the avatar's actual live emotion instead
        // of a permanently-static "Agent Active" label.
        let (_, pulse, emotion) = avatar.render_state(tick);
        let (status_color, status_label) = match emotion {
            crate::avatar::Emotion::Idle => (ACCENT_GREEN, "Idle"),
            crate::avatar::Emotion::Thinking => (ACCENT_AMBER, "Thinking…"),
            crate::avatar::Emotion::Speaking => (ACCENT_GREEN, "Speaking"),
            crate::avatar::Emotion::Alert => (ACCENT_RED, "Alert"),
            crate::avatar::Emotion::Happy => (0x34d399, "Happy"),
            crate::avatar::Emotion::Processing => (ACCENT_CYAN, "Processing"),
        };
        let dot_color = lerp_color(0x201a14, status_color, 0.5 + pulse * 0.5);
        self.fill_circle(150, STATUSBAR_HEIGHT / 2, 4, dot_color);
        self.draw_text(status_label, 160, 8, 10.0, TEXT_SECONDARY, false);

        // Right cluster, menu-bar-extra style: CPU · MEM · date · time.
        let now = chrono::Local::now();
        let time_str = now.format("%H:%M:%S").to_string();
        let date_str = now.format("%a %d %b").to_string();
        let cpu_str = format!("CPU {:.0}%", self.smooth_cpu);
        let mem_str = format!("MEM {:.0}%", self.smooth_mem_pct * 100.0);

        // Items drop off progressively (CPU, then MEM, then date) when the
        // window is too narrow for the whole cluster, instead of overlapping
        // the left side — the clock always stays.
        let left_edge = 300i32;

        let time_x = w as i32 - 66;
        self.draw_text(&time_str, time_x, 8, 10.5, TEXT_PRIMARY, false);

        let date_x = time_x - 78;
        if date_x > left_edge {
            self.draw_text(&date_str, date_x, 8, 9.5, TEXT_DIM, false);
            self.fill_rect((date_x - 12) as u32, 7, 1, STATUSBAR_HEIGHT - 14, BORDER);
        }

        let mem_x = date_x - 70;
        if mem_x > left_edge {
            let mem_color = lerp_color(ACCENT_GREEN, ACCENT_RED, self.smooth_mem_pct);
            self.draw_text(&mem_str, mem_x, 8, 9.5, mem_color, false);
        }

        let cpu_x = mem_x - 64;
        if cpu_x > left_edge {
            let cpu_color = lerp_color(ACCENT_GREEN, ACCENT_RED, self.smooth_cpu / 100.0);
            self.draw_text(&cpu_str, cpu_x, 8, 9.5, cpu_color, false);
        }
    }

    // ── Task Plan Panel ───────────────────────────────────────────────────────

    fn draw_plan_panel(&mut self, tick: u64) {
        if self.plan_steps.is_empty() {
            return;
        }

        let panel_w: u32 = 220;
        let step_h: u32 = 26;
        let header_h: u32 = 24;
        let padding: u32 = 8;
        let margin: u32 = 16;
        let panel_h = header_h + self.plan_steps.len() as u32 * step_h + padding;

        let x = margin;
        let y = STATUSBAR_HEIGHT + margin;

        // Background
        for py in y..y + panel_h {
            for px in x..x + panel_w {
                if px < self.width && py < self.height {
                    let idx = (py * self.width + px) as usize;
                    self.buf[idx] = blend_pixel(self.buf[idx], 0x080814, 0.82);
                }
            }
        }

        // Border
        self.fill_rect(x, y, panel_w, 1, BORDER);
        self.fill_rect(x, y, 1, panel_h, BORDER);
        self.fill_rect(x + panel_w - 1, y, 1, panel_h, BORDER);
        self.fill_rect(x, y + panel_h - 1, panel_w, 1, BORDER);
        self.fill_rect(x, y, panel_w, 2, ACCENT_BLUE);

        self.draw_text(
            "AGENT PLAN",
            x as i32 + 8,
            y as i32 + 6,
            9.0,
            TEXT_DIM,
            true,
        );
        self.fill_rect(x, y + header_h - 1, panel_w, 1, 0x201a14);

        let steps = self.plan_steps.clone();
        for (i, step) in steps.iter().enumerate() {
            let sy = y + header_h + i as u32 * step_h;
            let sy_i = sy as i32;

            let (dot_color, text_color) = match step.status {
                StepStatus::Pending => (TEXT_DIM, TEXT_DIM),
                StepStatus::Active => {
                    let pulse = (tick as f32 * 0.12).sin() * 0.5 + 0.5;
                    (lerp_color(ACCENT_BLUE, ACCENT_CYAN, pulse), TEXT_PRIMARY)
                }
                StepStatus::Done => (ACCENT_GREEN, TEXT_SECONDARY),
                StepStatus::Failed => (ACCENT_RED, 0xfca5a5),
            };

            let prefix = match step.status {
                StepStatus::Pending => "○",
                StepStatus::Active => "◉",
                StepStatus::Done => "✓",
                StepStatus::Failed => "✗",
            };

            self.fill_circle(x + 12, sy + step_h / 2, 3, dot_color);
            let label = format!("{} {}", prefix, truncate_str(&step.description, 26));
            self.draw_text(&label, x as i32 + 22, sy_i + 8, 8.5, text_color, false);

            // Connector line between steps
            if i + 1 < steps.len() {
                self.fill_rect(x + 11, sy + step_h - 2, 2, 4, BORDER);
            }
        }
    }

    // ── Cards ─────────────────────────────────────────────────────────────────

    fn draw_cards(&mut self, cards: &[&Card], _tick: u64) {
        let card_w: u32 = 340;
        let card_h: u32 = 115;
        let margin: u32 = 16;
        let start_x = self.width.saturating_sub(card_w + margin);
        let start_y = STATUSBAR_HEIGHT + margin;

        for (i, card) in cards.iter().enumerate() {
            let y = start_y + i as u32 * (card_h + 12);
            if y + card_h > self.height {
                break;
            }

            // Slide-in animation
            let slide_progress = (card.age_ticks as f32 / 20.0).min(1.0);
            let slide_offset = ((1.0 - ease_out_cubic(slide_progress)) * 70.0) as u32;
            let x = start_x + slide_offset;

            // Fade out near end of life
            let fade = if card.age_ticks > card.max_age.saturating_sub(60) {
                (card.max_age.saturating_sub(card.age_ticks)) as f32 / 60.0
            } else {
                1.0
            };
            if fade <= 0.0 {
                continue;
            }

            let (bg, accent) = match card.card_type {
                CardType::Info => (CARD_BG_INFO, ACCENT_PRIMARY),
                CardType::Task => (CARD_BG_TASK, ACCENT_BLUE),
                CardType::Alert => (CARD_BG_ALERT, ACCENT_RED),
                CardType::Weather => (CARD_BG_WEATHER, ACCENT_CYAN),
                CardType::Metric => (CARD_BG_METRIC, ACCENT_AMBER),
                CardType::Plan => (CARD_BG_PLAN, ACCENT_GREEN),
            };

            // Glassmorphism card body
            for py in y..y + card_h {
                for px in x..x + card_w {
                    if px < self.width && py < self.height {
                        let idx = (py * self.width + px) as usize;
                        self.buf[idx] = blend_pixel(self.buf[idx], bg, 0.88 * fade);
                    }
                }
            }

            // Accent left border
            self.fill_rect(x, y, 3, card_h, blend(accent, 0x000000, 1.0 - fade));
            // Top highlight
            self.fill_rect(x + 3, y, card_w - 3, 1, blend(bg, 0xffffff, 0.10));
            // Bottom shadow
            self.fill_rect(x, y + card_h - 1, card_w, 1, 0x000000);
            // Corner accent dot
            self.fill_circle(x + card_w - 14, y + 14, 5, blend(accent, 0x000000, 0.3));

            // Title
            self.draw_text(
                &card.title,
                (x + 14) as i32,
                (y + 14) as i32,
                12.5,
                blend(TEXT_PRIMARY, 0x000000, 1.0 - fade),
                true,
            );

            // Body text (word-wrap, 3 lines)
            let body_lines = wrap_text(&card.body, 38);
            for (li, line) in body_lines.iter().enumerate().take(3) {
                self.draw_text(
                    line,
                    (x + 14) as i32,
                    (y + 34 + li as u32 * 18) as i32,
                    10.0,
                    blend(TEXT_SECONDARY, 0x000000, 1.0 - fade),
                    false,
                );
            }

            // Progress bar
            let progress = 1.0 - (card.age_ticks as f32 / card.max_age as f32);
            let bar_w = ((card_w - 6) as f32 * progress) as u32;
            self.fill_rect(
                x + 3,
                y + card_h - 3,
                bar_w,
                2,
                blend(accent, 0x000000, 0.25),
            );
        }
    }

    // ── Avatar ───────────────────────────────────────────────────────────────

    /// Renders ZuuposOS's "buddy" character — a soft, lit orb creature with a
    /// pulsing antenna, expressive eyes and a continuous mouth curve instead
    /// of hard-edged debug shapes. Everything is drawn with anti-aliased
    /// primitives (`fill_circle_aa`/`fill_ellipse_aa`) so it reads as a
    /// character at a glance rather than pixel-art placeholder geometry.
    fn draw_avatar(&mut self, avatar: &Avatar, tick: u64) {
        let size = 96.0f32;
        let r = size / 2.0;
        // The character roams (see Avatar::tick) instead of sitting fixed in
        // the corner — this just reads its current position each frame.
        let (base_cx, base_cy) = avatar.position();
        let lean = if avatar.facing_right() {
            1.0f32
        } else {
            -1.0f32
        };

        let (blink, pulse, emotion) = avatar.render_state(tick);

        // Idle "breathing" bob — subtle even at rest so the character never
        // looks frozen. Speaking/Alert breathe faster and shallower (alert).
        let (breathe_freq, breathe_amp) = match emotion {
            crate::avatar::Emotion::Alert => (0.12, 1.2),
            crate::avatar::Emotion::Speaking => (0.10, 1.6),
            _ => (0.035, 2.6),
        };
        let breathe = (tick as f32 * breathe_freq).sin() * breathe_amp;
        let cx = base_cx;
        let cy = base_cy + breathe;

        let eye_color = match emotion {
            crate::avatar::Emotion::Idle => AVATAR_EYE,
            crate::avatar::Emotion::Thinking => ACCENT_AMBER,
            crate::avatar::Emotion::Speaking => ACCENT_GREEN,
            crate::avatar::Emotion::Alert => ACCENT_RED,
            crate::avatar::Emotion::Happy => 0x34d399,
            crate::avatar::Emotion::Processing => ACCENT_CYAN,
        };

        // Ground shadow — anchored (doesn't bob), squashes slightly as the
        // body rises, which sells the floating motion.
        let shadow_ry = 6.0f32;
        let shadow_rx = r * 0.78 - breathe * 0.9;
        self.fill_ellipse_aa(
            cx,
            base_cy + r + 10.0,
            shadow_rx.max(4.0),
            shadow_ry,
            0x000000,
            0.30,
        );

        // Antenna — thin stalk + glowing tip, pulses with the same rhythm as
        // the halo so the whole character reads as one living thing.
        // Leans a couple px toward the direction of travel while wandering —
        // a tiny cue that reads as "walking that way" rather than sliding.
        let wandering = matches!(
            emotion,
            crate::avatar::Emotion::Idle | crate::avatar::Emotion::Happy
        );
        let antenna_lean = if wandering { lean * 3.0 } else { 0.0 };
        let antenna_base_y = cy - r + 2.0;
        let antenna_tip_y = antenna_base_y - 14.0 - pulse * 2.0;
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let x = cx + antenna_lean * t;
            let y = antenna_base_y + (antenna_tip_y - antenna_base_y) * t;
            self.fill_circle_aa(x, y, 1.9, AVATAR_GLOW, 1.0);
        }
        let tip_glow = lerp_color(eye_color, 0xffffff, 0.15);
        let tip_x = cx + antenna_lean;
        self.fill_circle_aa(tip_x, antenna_tip_y, 5.0 + pulse * 1.5, eye_color, 0.22);
        self.fill_circle_aa(tip_x, antenna_tip_y, 2.6, tip_glow, 1.0);

        // Outer halo + inner glow ring (now AA, matched to emotion color
        // rather than always the generic purple AVATAR_GLOW).
        let glow_tint = lerp_color(AVATAR_GLOW, eye_color, 0.35);
        self.fill_circle_aa(cx, cy, r + 16.0, glow_tint, pulse * 0.22);
        self.fill_circle_aa(cx, cy, r + 6.0, glow_tint, pulse * 0.40);

        // Body — base sphere, then an offset lighter patch and a small
        // specular highlight to fake directional lighting (top-left light
        // source), which is what makes it read as an orb instead of a disc.
        self.fill_circle_aa(cx, cy, r, AVATAR_BG, 1.0);
        self.fill_circle_aa(cx - r * 0.12, cy - r * 0.12, r * 0.92, AVATAR_FACE, 1.0);
        self.fill_ellipse_aa(
            cx - r * 0.38,
            cy - r * 0.45,
            r * 0.30,
            r * 0.20,
            0xffffff,
            0.07,
        );
        // Bottom ambient occlusion — subtle darkening to ground the sphere.
        self.fill_ellipse_aa(cx, cy + r * 0.55, r * 0.85, r * 0.35, 0x000000, 0.10);

        // Eyes
        let eye_y = cy - 12.0;
        let left_x = cx - 15.0;
        let right_x = cx + 15.0;
        let eye_ry = if blink { 1.2 } else { 9.0 };
        for ex in [left_x, right_x] {
            self.fill_ellipse_aa(ex, eye_y, 6.5, eye_ry, eye_color, 1.0);
            if !blink {
                self.fill_circle_aa(ex, eye_y, 2.6, blend(eye_color, 0x000000, 0.55), 1.0);
                // Catchlight — tiny highlight dot, the single biggest thing
                // that makes a drawn eye look alive instead of a bullet point.
                self.fill_circle_aa(ex - 2.2, eye_y - 2.8, 1.4, 0xffffff, 0.85);
            }
        }

        // Emotion-specific brows / cheeks, layered under the mouth.
        match emotion {
            crate::avatar::Emotion::Alert => {
                for (ex, tilt) in [(left_x, 1.0f32), (right_x, -1.0f32)] {
                    for i in -2..=2 {
                        let fx = ex + i as f32 * 2.0;
                        let fy = eye_y - 9.0 - (i as f32 * tilt) * 0.8;
                        self.fill_circle_aa(fx, fy, 1.4, ACCENT_RED, 1.0);
                    }
                }
            }
            crate::avatar::Emotion::Happy => {
                let blush = blend(0xff6b6b, AVATAR_FACE, 0.55);
                self.fill_ellipse_aa(left_x - 7.0, eye_y + 8.0, 4.5, 2.8, blush, 0.8);
                self.fill_ellipse_aa(right_x + 7.0, eye_y + 8.0, 4.5, 2.8, blush, 0.8);
            }
            _ => {}
        }

        // Mouth — one continuous curve for every state instead of mismatched
        // rects/arcs; `curve` controls smile (+) vs. worried frown (-).
        let mouth_y = cy + 15.0;
        match emotion {
            crate::avatar::Emotion::Thinking | crate::avatar::Emotion::Processing => {
                self.draw_arc_mouth(cx, mouth_y, 7.0, 1.0, 1.4, eye_color);
                // Small bouncing "thinking" dots, offset to the upper-right
                // of the head so they don't collide with the antenna.
                for i in 0..3 {
                    let phase = (tick as f32 * 0.18 + i as f32 * 1.1).sin();
                    let dot_y = cy - r - 4.0 - phase * 3.0;
                    let dot_x = cx + r * 0.55 + i as f32 * 8.0;
                    let dot_color = lerp_color(AVATAR_FACE, eye_color, phase * 0.5 + 0.5);
                    self.fill_circle_aa(dot_x, dot_y, 2.2, dot_color, 1.0);
                }
            }
            crate::avatar::Emotion::Speaking => {
                self.draw_waveform_mouth(cx, cy, size as u32, tick);
            }
            crate::avatar::Emotion::Happy => {
                self.draw_arc_mouth(cx, mouth_y, 13.0, 7.0, 2.0, eye_color);
            }
            crate::avatar::Emotion::Alert => {
                self.draw_arc_mouth(cx, mouth_y, 9.0, -4.0, 1.6, eye_color);
            }
            crate::avatar::Emotion::Idle => {
                self.draw_arc_mouth(cx, mouth_y, 9.0, 3.0, 1.6, eye_color);
            }
        }
    }

    fn draw_waveform_mouth(&mut self, cx: f32, cy: f32, _size: u32, _tick: u64) {
        let base_y = cy + 15.0;
        let samples = self.waveform.len();
        let wave_w = 26.0f32;

        for i in 0..samples {
            let t = i as f32 / samples.max(1) as f32;
            let px = cx - wave_w / 2.0 + t * wave_w;
            let py = base_y - self.waveform[i] * 6.0;
            let color = lerp_color(ACCENT_GREEN, ACCENT_CYAN, t);
            self.fill_circle_aa(px, py, 1.8, color, 1.0);
        }
    }

    fn tick_waveform(&mut self, avatar: &Avatar, tick: u64) {
        let (_, _, emotion) = avatar.render_state(tick);
        let target_amp = if emotion == crate::avatar::Emotion::Speaking {
            1.0
        } else {
            0.0
        };
        let n = self.waveform.len();
        for (i, wave) in self.waveform.iter_mut().enumerate().take(n) {
            let phase = tick as f32 * 0.3 + i as f32 * 0.7;
            let target = (phase.sin() * 0.7 + (phase * 1.3).cos() * 0.3) * target_amp;
            *wave = *wave * 0.85 + target * 0.15;
        }
    }

    // ── Voice Dock ────────────────────────────────────────────────────────────

    /// Docked voice-input affordance on the right edge: a mic button with a
    /// breathing "ready" ring and the wake-word hint. This is a UI affordance
    /// only — there's no speech backend behind it yet (agentd's voice module
    /// is still whisper.cpp/TTS stubs), so it never claims to be listening,
    /// only ready.
    fn draw_voice_dock(&mut self, tick: u64, avatar: &Avatar) {
        let (_, _, emotion) = avatar.render_state(tick);
        let cx = self.width as f32 - 46.0;
        let cy = self.height as f32 - 120.0;
        let r = 22.0f32;

        let pulse = (tick as f32 * 0.035).sin() * 0.5 + 0.5;
        let speaking = emotion == crate::avatar::Emotion::Speaking;
        let ring_color = if speaking { ACCENT_GREEN } else { ACCENT_CYAN };

        // Breathing "ready" ring + button body.
        self.fill_circle_aa(
            cx,
            cy,
            r + 8.0 + pulse * 3.0,
            ring_color,
            0.08 + pulse * 0.05,
        );
        self.fill_circle_aa(cx, cy, r, 0x1a1510, 1.0);
        self.fill_circle_aa(cx, cy, r, ring_color, if speaking { 0.25 } else { 0.0 });

        // Minimal mic glyph: capsule head + stem + base.
        self.fill_ellipse_aa(cx, cy - 5.0, 5.5, 8.5, TEXT_PRIMARY, 0.92);
        self.fill_rect((cx - 1.0) as u32, (cy + 3.0) as u32, 2, 9, TEXT_PRIMARY);
        self.fill_rect((cx - 6.0) as u32, (cy + 12.0) as u32, 12, 2, TEXT_PRIMARY);

        self.draw_text(
            "\"Hey ZuuposOS\"",
            (cx - 48.0) as i32,
            (cy + r + 10.0) as i32,
            9.0,
            TEXT_SECONDARY,
            false,
        );
        self.draw_text(
            "tap to talk",
            (cx - 24.0) as i32,
            (cy + r + 24.0) as i32,
            8.0,
            TEXT_DIM,
            false,
        );
    }

    // ── Boot Overlay ──────────────────────────────────────────────────────────

    fn draw_boot_overlay(&mut self, tick: u64) {
        let alpha = 1.0 - (tick as f32 / 120.0).min(1.0);
        if alpha <= 0.01 {
            return;
        }

        // Dim overlay
        for px in self.buf.iter_mut() {
            let r = ((*px >> 16) & 0xff) as f32 / 255.0;
            let g = ((*px >> 8) & 0xff) as f32 / 255.0;
            let b = (*px & 0xff) as f32 / 255.0;
            let dr = r * (1.0 - alpha * 0.9);
            let dg = g * (1.0 - alpha * 0.9);
            let db = b * (1.0 - alpha * 0.9);
            *px = ((dr * 255.0) as u32) << 16 | ((dg * 255.0) as u32) << 8 | (db * 255.0) as u32;
        }

        let cx = (self.width / 2) as i32 - 80;
        let cy = (self.height / 2) as i32 - 20;

        let logo_color = blend(BG_BASE, ACCENT_PRIMARY, alpha);
        self.draw_text("ZUUPOS OS", cx, cy, 36.0, logo_color, true);

        let sub_color = blend(BG_BASE, TEXT_SECONDARY, alpha * 0.7);
        self.draw_text(
            "AI-Native Agentic Operating System",
            cx - 20,
            cy + 50,
            13.5,
            sub_color,
            false,
        );

        // Loading dots animated
        let dots_done = (tick / 25) % 6;
        let dot_str: String = (0..5)
            .map(|i| if i < dots_done { '●' } else { '○' })
            .collect();
        let dots_color = blend(BG_BASE, ACCENT_PRIMARY, alpha * 0.7);
        self.draw_text(&dot_str, cx + 60, cy + 76, 13.0, dots_color, false);
    }

    // ── Drawing Primitives ────────────────────────────────────────────────────

    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        if x >= self.width || y >= self.height {
            return;
        }
        for py in y..y2 {
            for px in x..x2 {
                self.buf[(py * self.width + px) as usize] = color & 0x00ff_ffff;
            }
        }
    }

    fn fill_circle(&mut self, cx: u32, cy: u32, r: u32, color: u32) {
        if r == 0 {
            return;
        }
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

    /// Anti-aliased filled circle with a soft ~1px edge feather, blended onto
    /// whatever is already in `buf`. Used by the avatar so it reads as a
    /// smooth character instead of the jagged pixel-art circles `fill_circle`
    /// produces at small sizes.
    fn fill_circle_aa(&mut self, cx: f32, cy: f32, r: f32, color: u32, alpha_mul: f32) {
        if r <= 0.0 || alpha_mul <= 0.0 {
            return;
        }
        if cx + r < 0.0 || cx - r > self.width as f32 || cy + r < 0.0 || cy - r > self.height as f32
        {
            return;
        }
        let x0 = (cx - r - 1.0).floor().max(0.0) as u32;
        let x1 = ((cx + r + 1.0).ceil() as i64).clamp(0, self.width as i64 - 1) as u32;
        let y0 = (cy - r - 1.0).floor().max(0.0) as u32;
        let y1 = ((cy + r + 1.0).ceil() as i64).clamp(0, self.height as i64 - 1) as u32;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let coverage = (r + 0.5 - dist).clamp(0.0, 1.0) * alpha_mul;
                if coverage <= 0.0 {
                    continue;
                }
                let idx = (py * self.width + px) as usize;
                self.buf[idx] = blend_pixel(self.buf[idx], color, coverage);
            }
        }
    }

    /// Anti-aliased filled ellipse (same technique as `fill_circle_aa`).
    /// Used for eyes, the ground shadow, and the specular highlight.
    fn fill_ellipse_aa(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: u32, alpha_mul: f32) {
        if rx <= 0.0 || ry <= 0.0 || alpha_mul <= 0.0 {
            return;
        }
        if cx + rx < 0.0
            || cx - rx > self.width as f32
            || cy + ry < 0.0
            || cy - ry > self.height as f32
        {
            return;
        }
        let x0 = (cx - rx - 1.0).floor().max(0.0) as u32;
        let x1 = ((cx + rx + 1.0).ceil() as i64).clamp(0, self.width as i64 - 1) as u32;
        let y0 = (cy - ry - 1.0).floor().max(0.0) as u32;
        let y1 = ((cy + ry + 1.0).ceil() as i64).clamp(0, self.height as i64 - 1) as u32;
        // Feather width in normalized ellipse-space, sized off the smaller axis
        // so the edge softness stays proportional on very thin ellipses (blinks).
        let aa = 1.0 / rx.min(ry).max(0.5);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let nx = (px as f32 + 0.5 - cx) / rx;
                let ny = (py as f32 + 0.5 - cy) / ry;
                let dist = (nx * nx + ny * ny).sqrt();
                let coverage = ((1.0 - dist) / aa).clamp(0.0, 1.0) * alpha_mul;
                if coverage <= 0.0 {
                    continue;
                }
                let idx = (py * self.width + px) as usize;
                self.buf[idx] = blend_pixel(self.buf[idx], color, coverage);
            }
        }
    }

    /// Stamps a smooth arc of small AA dots between `-half_width..half_width`
    /// around `cx`, shaped by a parabola: positive `curve` reads as a smile
    /// (corners pulled up relative to the center), negative as a frown/worried
    /// mouth, ~0 as a flat line.
    fn draw_arc_mouth(
        &mut self,
        cx: f32,
        base_y: f32,
        half_width: f32,
        curve: f32,
        thickness: f32,
        color: u32,
    ) {
        let steps = 16;
        for i in 0..=steps {
            let t = (i as f32 / steps as f32) * 2.0 - 1.0;
            let x = cx + t * half_width;
            let y = base_y + curve * (1.0 - t * t);
            self.fill_circle_aa(x, y, thickness, color, 1.0);
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
                if cov == 0 {
                    continue;
                }
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
                self.buf[idx] =
                    ((fr * 255.0) as u32) << 16 | ((fg * 255.0) as u32) << 8 | (fb * 255.0) as u32;
            }
            cursor_x += metrics.advance_width as i32;
        }
    }

    fn present(&mut self) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        let w = NonZeroU32::new(self.width).unwrap();
        let h = NonZeroU32::new(self.height).unwrap();
        surface.resize(w, h).unwrap();
        let mut frame = surface.buffer_mut().unwrap();
        frame.copy_from_slice(&self.buf);
        frame.present().unwrap();
    }
}

// ─── Helper Functions ─────────────────────────────────────────────────────────

fn spawn_particle(rng: &mut SmallRng, w: u32, h: u32, initial: bool) -> Particle {
    let x = rng.gen_range(0.0..w as f32);
    let y = if initial {
        rng.gen_range(0.0..h as f32)
    } else {
        h as f32 // spawn from bottom edges
    };
    let speed = rng.gen_range(0.1f32..0.5);
    let angle = rng.gen_range(std::f32::consts::PI * 1.2..std::f32::consts::PI * 1.8);
    let max_life = rng.gen_range(180.0f32..600.0);
    let size = rng.gen_range(0.8f32..3.0);
    let hue = rng.gen_range(0u32..3);
    let color = match hue {
        0 => ACCENT_PRIMARY, // warm amber dust
        1 => ACCENT_AMBER,
        _ => ACCENT_CYAN, // one cool counterpoint
    };
    Particle {
        x,
        y,
        vx: angle.cos() * speed,
        vy: angle.sin() * speed * 0.4 - 0.25, // slight upward drift
        life: 1.0,
        max_life,
        size,
        color,
    }
}

fn blend_pixel(bg: u32, fg: u32, alpha: f32) -> u32 {
    let a = alpha.clamp(0.0, 1.0);
    let br = ((bg >> 16) & 0xff) as f32;
    let bg_g = ((bg >> 8) & 0xff) as f32;
    let bb = (bg & 0xff) as f32;
    let fr = ((fg >> 16) & 0xff) as f32;
    let fg_g = ((fg >> 8) & 0xff) as f32;
    let fb = (fg & 0xff) as f32;
    let r = (fr * a + br * (1.0 - a)) as u32;
    let g = (fg_g * a + bg_g * (1.0 - a)) as u32;
    let b = (fb * a + bb * (1.0 - a)) as u32;
    (r << 16) | (g << 8) | b
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    }
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t1 = 1.0 - t;
    1.0 - t1 * t1 * t1
}

pub fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ar = (a >> 16) & 0xff;
    let ag = (a >> 8) & 0xff;
    let ab = a & 0xff;
    let br = (b >> 16) & 0xff;
    let bg_g = (b >> 8) & 0xff;
    let bb = b & 0xff;
    let r = (ar as f32 + (br as f32 - ar as f32) * t) as u32;
    let g = (ag as f32 + (bg_g as f32 - ag as f32) * t) as u32;
    let bv = (ab as f32 + (bb as f32 - ab as f32) * t) as u32;
    (r << 16) | (g << 8) | bv
}

pub fn blend(a: u32, b: u32, alpha: f32) -> u32 {
    lerp_color(b, a, alpha)
}

pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
