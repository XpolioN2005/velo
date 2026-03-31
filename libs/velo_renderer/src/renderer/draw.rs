use super::Renderer;
use crate::types::{rect::Rect, result_item::ResultItem};
use windows::Win32::Graphics::{
    Direct2D::{Common::*, D2D1_DRAW_TEXT_OPTIONS_NONE},
    DirectWrite::*,
};

impl Renderer {
    pub fn draw_rect(&self, rect: Rect) {
        unsafe {
            self.target.FillRectangle(
                &D2D_RECT_F {
                    left: rect.x,
                    top: rect.y,
                    right: rect.x + rect.w,
                    bottom: rect.y + rect.h,
                },
                &self.brush_highlight,
            );
        }
    }

    pub fn draw_text(&self, text: &str, x: f32, y: f32) {
        unsafe {
            let text_utf16: Vec<u16> = text.encode_utf16().collect();

            self.target.DrawText(
                &text_utf16,
                &self.text_format,
                &D2D_RECT_F {
                    left: x,
                    top: y,
                    right: x + 500.0,
                    bottom: y + 50.0,
                },
                &self.brush_text,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
    }

    pub fn draw_results(&self, items: &[ResultItem]) {
        let layout = self.compute_layout(items);

        for item in layout {
            if item.selected {
                self.draw_rect(item.rect);
            }

            self.draw_text(&item.text, item.text_pos.0, item.text_pos.1);
        }
    }
}
