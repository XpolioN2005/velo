use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
        },
        System::Com::*,
        UI::WindowsAndMessaging::GetClientRect,
    },
    core::*,
};

use crate::style::{dimensions::Dimensions, theme::Theme};

pub struct Renderer {
    pub(crate) hwnd: HWND,
    pub(crate) target: ID2D1HwndRenderTarget,

    // Stored to avoid recreating per frame
    pub(crate) dw_factory: IDWriteFactory,
    pub(crate) text_format: IDWriteTextFormat,

    pub(crate) brush_bg: ID2D1SolidColorBrush,
    pub(crate) brush_text: ID2D1SolidColorBrush,
    pub(crate) brush_highlight: ID2D1SolidColorBrush,
    pub(crate) brush_input_bg: ID2D1SolidColorBrush,
    pub(crate) brush_cursor: ID2D1SolidColorBrush,

    pub(crate) theme: Theme,
    pub(crate) dims: Dimensions,
    pub(crate) dpi: f32,
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

            let factory = Self::create_d2d_factory()?;
            let target = Self::create_render_target(&factory, hwnd)?;

            let dw_factory = Self::create_dwrite_factory()?;
            let text_format = Self::create_text_format(&dw_factory)?;

            let theme = Theme::default();

            let (brush_bg, brush_text, brush_highlight, brush_input_bg, brush_cursor) =
                Self::create_brushes(&target, &theme)?;

            let dpi = Self::get_dpi(&target);

            Ok(Self {
                hwnd,
                target,
                dw_factory,
                text_format,
                brush_bg,
                brush_text,
                brush_highlight,
                brush_input_bg,
                brush_cursor,
                theme,
                dims: Dimensions::default(),
                dpi,
            })
        }
    }

    fn create_d2d_factory() -> Result<ID2D1Factory> {
        unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }
    }

    fn create_render_target(factory: &ID2D1Factory, hwnd: HWND) -> Result<ID2D1HwndRenderTarget> {
        unsafe {
            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect);

            let size = D2D_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            };

            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: size,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let target = factory.CreateHwndRenderTarget(&rt_props, &hwnd_props)?;

            Ok(target)
        }
    }

    fn create_dwrite_factory() -> Result<IDWriteFactory> {
        unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }
    }

    fn create_text_format(factory: &IDWriteFactory) -> Result<IDWriteTextFormat> {
        unsafe {
            factory.CreateTextFormat(
                w!("Segoe UI"),
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                18.0,
                w!("en-us"),
            )
        }
    }

    fn create_brushes(
        target: &ID2D1HwndRenderTarget,
        theme: &Theme,
    ) -> Result<(
        ID2D1SolidColorBrush,
        ID2D1SolidColorBrush,
        ID2D1SolidColorBrush,
        ID2D1SolidColorBrush,
        ID2D1SolidColorBrush,
    )> {
        unsafe {
            Ok((
                target.CreateSolidColorBrush(&theme.bg, None)?,
                target.CreateSolidColorBrush(&theme.text, None)?,
                target.CreateSolidColorBrush(&theme.highlight, None)?,
                target.CreateSolidColorBrush(&theme.input_bg, None)?,
                target.CreateSolidColorBrush(&theme.cursor, None)?,
            ))
        }
    }

    fn get_dpi(target: &ID2D1HwndRenderTarget) -> f32 {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            target.GetDpi(&mut x, &mut y);
            x
        }
    }

    pub fn size(&self) -> (f32, f32) {
        unsafe {
            let s = self.target.GetSize();
            (s.width, s.height)
        }
    }
}
