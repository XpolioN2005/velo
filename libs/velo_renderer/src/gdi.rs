// gdi.rs
use crate::types::{Color, Point, Rect};
use std::collections::HashMap;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub struct GdiContext {
    hdc: HDC,
    brush_cache: HashMap<(u8, u8, u8), HBRUSH>,
}

impl GdiContext {
    pub fn new(hdc: HDC) -> Self {
        Self {
            hdc,
            brush_cache: HashMap::new(),
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        // reuse brushes to avoid per-frame allocation
        let brush = *self
            .brush_cache
            .entry((color.0, color.1, color.2))
            .or_insert_with(|| unsafe {
                CreateSolidBrush(windows::Win32::Foundation::COLORREF(rgb(
                    color.0, color.1, color.2,
                )))
            });

        unsafe {
            let r = RECT {
                left: rect.x,
                top: rect.y,
                right: rect.x + rect.w,
                bottom: rect.y + rect.h,
            };
            FillRect(self.hdc, &r, brush);
        }
    }

    pub fn draw_text(&mut self, pos: Point, text: &str) {
        unsafe {
            SetBkMode(self.hdc, TRANSPARENT);
            let _ = TextOutW(self.hdc, pos.x, pos.y, &to_wide(text));
        }
    }
}

// Helper: convert &str to Vec<u16> (null-terminated)
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
