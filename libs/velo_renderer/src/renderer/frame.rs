use super::Renderer;
use windows::Win32::Graphics::Direct2D::Common::*;

impl Renderer {
    pub fn begin(&self) {
        unsafe {
            self.target.BeginDraw();
            self.target.Clear(Some(&self.theme.bg));
        }
    }

    pub fn end(&self) {
        unsafe {
            self.target.EndDraw(None, None).ok();
        }
    }

    pub fn resize(&self, w: u32, h: u32) {
        unsafe {
            self.target
                .Resize(&D2D_SIZE_U {
                    width: w,
                    height: h,
                })
                .ok();
        }
    }
}
