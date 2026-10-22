//! UEFI GOP framebuffer driver.
//! Provides a pixel-addressable display surface.

use super::super::arch::x86_64::boot_info::BootInfo;
use spinning_top::Spinlock;

pub struct Framebuffer {
    pub base: *mut u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: u32,
}

// SAFETY: We guarantee single-threaded access via Spinlock.
unsafe impl Send for Framebuffer {}

static FB: Spinlock<Option<Framebuffer>> = Spinlock::new(None);

pub fn init(boot_info: &BootInfo) {
    *FB.lock() = Some(Framebuffer {
        base: boot_info.fb_addr as *mut u32,
        width: boot_info.fb_width,
        height: boot_info.fb_height,
        stride: boot_info.fb_stride,
        pixel_format: boot_info.fb_pixel_format,
    });
    log::debug!(
        "[drv::fb] {}x{} framebuffer @ {:#x}",
        boot_info.fb_width,
        boot_info.fb_height,
        boot_info.fb_addr
    );
}

/// Write a pixel at (x, y) with the given 0x00RRGGBB color.
pub fn put_pixel(x: u32, y: u32, color: u32) {
    let guard = FB.lock();
    if let Some(ref fb) = *guard {
        if x < fb.width && y < fb.height {
            unsafe {
                fb.base
                    .add((y * fb.stride + x) as usize)
                    .write_volatile(color);
            }
        }
    }
}

/// Fill a rectangle with a solid color.
pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            put_pixel(x + dx, y + dy, color);
        }
    }
}

/// Clear the entire screen.
pub fn clear(color: u32) {
    let guard = FB.lock();
    if let Some(ref fb) = *guard {
        let total = fb.stride * fb.height;
        unsafe {
            for i in 0..total {
                fb.base.add(i as usize).write_volatile(color);
            }
        }
    }
}
