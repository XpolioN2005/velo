use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
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

    pub(crate) text_format: IDWriteTextFormat,

    pub(crate) brush_bg: ID2D1SolidColorBrush,
    pub(crate) brush_text: ID2D1SolidColorBrush,
    pub(crate) brush_highlight: ID2D1SolidColorBrush,

    pub(crate) theme: Theme,
    pub(crate) dims: Dimensions,

    pub(crate) dpi: f32,
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            let size = D2D_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            };

            let target = factory.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES::default(),
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd,
                    pixelSize: size,
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )?;

            target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            target.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);

            let dw_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            let text_format = dw_factory.CreateTextFormat(
                w!("Segoe UI"),
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                18.0,
                w!("en-us"),
            )?;

            let theme = Theme::default();

            let brush_bg = target.CreateSolidColorBrush(&theme.bg, None)?;
            let brush_text = target.CreateSolidColorBrush(&theme.text, None)?;
            let brush_highlight = target.CreateSolidColorBrush(&theme.highlight, None)?;

            let mut dpi_x = 0.0;
            let mut dpi_y = 0.0;
            target.GetDpi(&mut dpi_x, &mut dpi_y);

            Ok(Self {
                hwnd,
                target,
                text_format,
                brush_bg,
                brush_text,
                brush_highlight,
                theme,
                dims: Dimensions::default(),
                dpi: dpi_x,
            })
        }
    }
}
