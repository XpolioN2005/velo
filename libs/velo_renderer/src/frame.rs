// frame.rs
use crate::gdi::GdiContext;
use crate::types::{Color, Point, Rect};

pub struct Frame<'a> {
    pub(crate) gdi: &'a mut GdiContext,
}

impl<'a> Frame<'a> {
    /// Draw a filled rectangle
    pub fn rect(&mut self, rect: Rect, color: Color) {
        self.gdi.fill_rect(rect, color);
    }

    /// Draw text
    pub fn text(&mut self, pos: Point, text: &str) {
        self.gdi.draw_text(pos, text);
    }
}
