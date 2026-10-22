//! Software renderer — writes pixels to the softbuffer framebuffer.
//! On bare metal, this will write directly to the GOP framebuffer.

use crate::avatar::Avatar;
use crate::cards::{Card, CardType};
use softbuffer::Surface;
use std::num::NonZeroU32;
use winit::window::Window;
use std::rc::Rc;

pub struct Renderer {
    surface: Surface<Rc<Window>, Rc<Window>>,
    pub width: u32,
    pub height: u32,
    buffer: Vec<u32>,
}

impl Renderer {
    pub fn new(window: &Rc<Window>, width: u32, height: u32) -> Self {
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        Self {
            surface,
            width,
            height,
            buffer: vec![0u32; (width * height) as usize],
        }
    }

    pub fn begin_frame(&mut self) {
        // Clear buffer
        self.buffer.fill(0x000d_0d1a); // Deep dark navy
    }

    /// Draw an animated gradient background with subtle noise.
    pub fn draw_background(&mut self, tick: u64) {
        let w = self.width as f32;
        let h = self.height as f32;
        let t = tick as f32 * 0.002;

        for y in 0..self.height {
            for x in 0..self.width {
                // Animated radial gradient: deep purple-navy
                let nx = x as f32 / w - 0.5;
                let ny = y as f32 / h - 0.5;
                let dist = (nx * nx + ny * ny).sqrt();
                let wave = (dist * 6.0 - t).sin() * 0.5 + 0.5;

                let r = ((0.05 + wave * 0.04) * 255.0) as u32;
                let g = ((0.03 + wave * 0.02) * 255.0) as u32;
                let b = ((0.12 + wave * 0.08) * 255.0) as u32;

                self.buffer[(y * self.width + x) as usize] = (r << 16) | (g << 8) | b;
            }
        }
    }

    /// Draw floating cards from top-right corner.
    pub fn draw_cards(&mut self, cards: &[&Card]) {
        let card_w = 380u32;
        let card_h = 120u32;
        let margin = 20u32;
        let start_x = self.width - card_w - margin;

        for (i, card) in cards.iter().enumerate() {
            let y = margin + i as u32 * (card_h + 12);
            let x = start_x;

            // Card background with glassmorphism (semi-transparent dark purple)
            let bg = match card.card_type {
                CardType::Info => 0x1a1a2e,
                CardType::Task => 0x16213e,
                CardType::Alert => 0x2d1b1b,
                CardType::Weather => 0x0f3460,
            };
            let accent = match card.card_type {
                CardType::Info => 0x7c3aed,
                CardType::Task => 0x2563eb,
                CardType::Alert => 0xdc2626,
                CardType::Weather => 0x0ea5e9,
            };

            self.fill_rect(x, y, card_w, card_h, bg);
            // Accent left border
            self.fill_rect(x, y, 4, card_h, accent);
            // Subtle highlight at top
            self.fill_rect(x + 4, y, card_w - 4, 1, 0x333355);

            // Title text (simplified — just color the first 2 rows)
            self.fill_rect(x + 16, y + 14, 200, 3, 0xccccff); // Title placeholder
            // Body text
            self.fill_rect(x + 16, y + 28, 300, 2, 0x7777aa); // Body line 1
            self.fill_rect(x + 16, y + 36, 260, 2, 0x7777aa); // Body line 2
        }
    }

    /// Draw the Taby-style AI avatar in the bottom-left corner.
    pub fn draw_avatar(&mut self, avatar: &Avatar) {
        let size = 80u32;
        let x = 24u32;
        let y = self.height - size - 24;

        let (blink, pulse) = avatar.state();

        // Outer glow (animated purple ring)
        let glow_alpha = ((pulse * std::f32::consts::PI).sin().abs() * 60.0) as u32;
        let glow_color = 0x00_4c_1a_8f | (glow_alpha << 24);
        self.fill_circle(x + size / 2, y + size / 2, size / 2 + 8, 0x7c3aed & 0x00ffffff | (glow_alpha << 24));

        // Avatar face background (rounded square — approximated as square)
        self.fill_rect(x, y, size, size, 0x1e1b4b);
        // Face circle
        self.fill_circle(x + size / 2, y + size / 2, size / 2 - 4, 0x2d2a6e);

        // Eyes
        let eye_y = y + size / 2 - 8;
        let eye_left_x = x + size / 2 - 12;
        let eye_right_x = x + size / 2 + 12;
        let eye_h = if blink { 1u32 } else { 6u32 };
        let eye_w = 8u32;
        // Left eye
        self.fill_rect(eye_left_x - eye_w / 2, eye_y - eye_h / 2, eye_w, eye_h, 0x7c3aed);
        // Right eye
        self.fill_rect(eye_right_x - eye_w / 2, eye_y - eye_h / 2, eye_w, eye_h, 0x7c3aed);

        // Smile
        let smile_y = y + size / 2 + 8;
        let smile_x = x + size / 2 - 12;
        self.fill_rect(smile_x, smile_y, 24, 3, 0x7c3aed);
        self.fill_rect(smile_x, smile_y - 2, 3, 5, 0x7c3aed);
        self.fill_rect(smile_x + 21, smile_y - 2, 3, 5, 0x7c3aed);
    }

    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        for py in y..y2 {
            for px in x..x2 {
                self.buffer[(py * self.width + px) as usize] = color & 0x00ffffff;
            }
        }
    }

    fn fill_circle(&mut self, cx: u32, cy: u32, r: u32, color: u32) {
        let r2 = (r * r) as i64;
        let cx = cx as i64;
        let cy = cy as i64;
        let x0 = (cx - r as i64).max(0) as u32;
        let x1 = (cx + r as i64).min(self.width as i64 - 1) as u32;
        let y0 = (cy - r as i64).max(0) as u32;
        let y1 = (cy + r as i64).min(self.height as i64 - 1) as u32;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let dx = px as i64 - cx;
                let dy = py as i64 - cy;
                if dx * dx + dy * dy <= r2 {
                    self.buffer[(py * self.width + px) as usize] = color & 0x00ffffff;
                }
            }
        }
    }

    pub fn end_frame(&mut self, window: &Window) {
        let (w, h) = (self.width, self.height);
        self.surface
            .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
            .unwrap();
        let mut buf = self.surface.buffer_mut().unwrap();
        buf.copy_from_slice(&self.buffer);
        buf.present().unwrap();
    }
}
