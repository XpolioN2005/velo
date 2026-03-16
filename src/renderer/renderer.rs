use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct2D::Common::{D2D_SIZE_U, D2D1_ALPHA_MODE_UNKNOWN, D2D1_PIXEL_FORMAT},
    Win32::Graphics::Direct2D::{
        D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
        D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES, D2D1CreateFactory, ID2D1Factory,
        ID2D1HwndRenderTarget,
    },
    Win32::Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWriteCreateFactory, IDWriteFactory,
    },
    Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
    Win32::UI::WindowsAndMessaging::GetClientRect,
    core::*,
};

use super::draw::TextFormat;
use super::theme;

pub struct Renderer {
    pub factory: ID2D1Factory,
    pub target: ID2D1HwndRenderTarget,
    pub dwrite: IDWriteFactory,
    pub text_ui: TextFormat,
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc)?;

            let size = D2D_SIZE_U {
                width: (rc.right - rc.left) as u32,
                height: (rc.bottom - rc.top) as u32,
            };

            let rtp = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: Default::default(),
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_UNKNOWN,
                    alphaMode: D2D1_ALPHA_MODE_UNKNOWN,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: Default::default(),
                minLevel: Default::default(),
            };

            let hwnd_rtp = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: size,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let target = factory.CreateHwndRenderTarget(&rtp, &hwnd_rtp)?;

            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            let text_ui = TextFormat::new(&dwrite, 14.0)?;

            Ok(Self {
                factory,
                target,
                dwrite,
                text_ui,
            })
        }
    }

    pub fn begin(&self) {
        unsafe { self.target.BeginDraw() }
    }

    pub fn end(&self) -> Result<()> {
        unsafe { self.target.EndDraw(None, None) }
    }

    pub fn clear(&self) {
        unsafe { self.target.Clear(Some(&theme::BG)) }
    }
}
