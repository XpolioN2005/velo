use windows::{
    Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    Win32::Graphics::Direct2D::Common::D2D1_COLOR_F,
    Win32::Graphics::Direct2D::{
        D2D1_BRUSH_PROPERTIES, ID2D1HwndRenderTarget, ID2D1SolidColorBrush,
    },
    Win32::Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_REGULAR, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
        DWRITE_TEXT_ALIGNMENT_LEADING, DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat,
    },
    core::*,
};

/// A text format — font, size, alignment. Create once, reuse every frame.
pub struct TextFormat {
    pub format: IDWriteTextFormat,
}

impl TextFormat {
    pub fn new(dwrite: &IDWriteFactory, size: f32) -> Result<Self> {
        unsafe {
            let format = dwrite.CreateTextFormat(
                w!("Segoe UI"),
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("en-us"),
            )?;

            // Text left-aligned, vertically centered
            format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING)?;
            format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;

            Ok(Self { format })
        }
    }
}

/// Draw a string into a rect using the given format and color.
pub fn draw_text(
    target: &ID2D1HwndRenderTarget,
    text: &str,
    fmt: &TextFormat,
    rect: D2D_RECT_F,
    color: D2D1_COLOR_F,
) -> Result<()> {
    unsafe {
        // Create a solid color brush — cheap, done per draw call for now
        let brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&color, None)?;

        // Convert &str to UTF-16 for Win32
        let wide: Vec<u16> = text.encode_utf16().collect();

        target.DrawText(
            &wide,
            &fmt.format,
            &rect,
            &brush,
            Default::default(),
            Default::default(),
        );

        Ok(())
    }
}
