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
    pub fn measure_text_until(&self, text: &str, pos: usize) -> f32 {
        unsafe {
            let sub = &text[..pos];
            let utf16: Vec<u16> = sub.encode_utf16().collect();

            let layout = self
                .dw_factory
                .CreateTextLayout(&utf16, &self.text_format, 1000.0, 100.0)
                .unwrap();

            let mut metrics = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut metrics).unwrap();

            metrics.width
        }
    }
    pub fn draw_input_bar(&self, text: &str, cursor_pos: usize) {
        let (width, _) = self.size();

        // Background
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: self.dims.input_height,
        };

        unsafe {
            self.target.FillRectangle(
                &D2D_RECT_F {
                    left: rect.x,
                    top: rect.y,
                    right: rect.x + rect.w,
                    bottom: rect.y + rect.h,
                },
                &self.brush_input_bg,
            );
        }

        // Text
        let text_x = 10.0;
        let text_y = 10.0;

        self.draw_text(text, text_x, text_y);

        // Cursor
        let cursor_x = text_x + self.measure_text_until(text, cursor_pos);

        let cursor_rect = Rect {
            x: cursor_x,
            y: 8.0,
            w: 2.0,
            h: self.dims.input_height - 16.0,
        };

        unsafe {
            self.target.FillRectangle(
                &D2D_RECT_F {
                    left: cursor_rect.x,
                    top: cursor_rect.y,
                    right: cursor_rect.x + cursor_rect.w,
                    bottom: cursor_rect.y + cursor_rect.h,
                },
                &self.brush_cursor,
            );
        }
    }
}
