// renderer.rs
use crate::frame::Frame;
use crate::gdi::GdiContext;
use crate::types::{Color, Point, Rect};
use windows::Win32::Graphics::Gdi::HDC;
pub struct Renderer {
    gdi: GdiContext,
}

impl Renderer {
    /// Create a renderer from an existing HDC
    pub fn new(hdc: HDC) -> Self {
        Self {
            gdi: GdiContext::new(hdc),
        }
    }

    pub fn begin_frame(&mut self) -> Frame {
        Frame { gdi: &mut self.gdi }
    }

    pub fn end_frame(&mut self) {}
}
