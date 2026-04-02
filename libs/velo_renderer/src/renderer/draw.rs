use windows::Win32::Graphics::{
    Direct2D::{
        Common::*, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ROUNDED_RECT, ID2D1HwndRenderTarget,
        ID2D1SolidColorBrush,
    },
    DirectWrite::*,
};
use windows_numerics::Vector2;

// ---------- RECT ----------

pub fn rect_filled(target: &ID2D1HwndRenderTarget, rect: D2D_RECT_F, brush: &ID2D1SolidColorBrush) {
    unsafe {
        target.FillRectangle(&rect, brush);
    }
}

pub fn rect_rounded(
    target: &ID2D1HwndRenderTarget,
    rect: D2D_RECT_F,
    brush: &ID2D1SolidColorBrush,
    radius: f32,
) {
    let rounded = D2D1_ROUNDED_RECT {
        rect,
        radiusX: radius,
        radiusY: radius,
    };

    unsafe {
        target.FillRoundedRectangle(&rounded, brush);
    }
}

// ---------- TEXT ----------

pub fn text(
    target: &ID2D1HwndRenderTarget,
    text: &str,
    format: &IDWriteTextFormat,
    rect: D2D_RECT_F,
    brush: &ID2D1SolidColorBrush,
) {
    let utf16: Vec<u16> = text.encode_utf16().collect();

    unsafe {
        target.DrawText(
            &utf16,
            format,
            &rect,
            brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
    }
}

// ---------- TEXT LAYOUT ----------

pub fn create_text_layout(
    dwrite: &IDWriteFactory,
    text: &str,
    format: &IDWriteTextFormat,
    width: f32,
    height: f32,
) -> Option<IDWriteTextLayout> {
    let utf16: Vec<u16> = text.encode_utf16().collect();

    unsafe { dwrite.CreateTextLayout(&utf16, format, width, height).ok() }
}

pub fn draw_text_layout(
    target: &ID2D1HwndRenderTarget,
    layout: &IDWriteTextLayout,
    x: f32,
    y: f32,
    brush: &ID2D1SolidColorBrush,
) {
    unsafe {
        target.DrawTextLayout(
            Vector2 { X: x, Y: y },
            layout,
            brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
        );
    }
}

// ---------- TEXT HIGHLIGHT ----------

pub fn text_highlighted(
    target: &ID2D1HwndRenderTarget,
    dwrite: &IDWriteFactory,
    text: &str,
    format: &IDWriteTextFormat,
    rect: D2D_RECT_F,
    normal: &ID2D1SolidColorBrush,
    highlight: &ID2D1SolidColorBrush,
    indices: &[usize],
) {
    let layout = match create_text_layout(
        dwrite,
        text,
        format,
        rect.right - rect.left,
        rect.bottom - rect.top,
    ) {
        Some(l) => l,
        None => return,
    };

    unsafe {
        for &i in indices {
            let _ = layout.SetDrawingEffect(
                Some((&highlight.clone()).into()),
                DWRITE_TEXT_RANGE {
                    startPosition: i as u32,
                    length: 1,
                },
            );
        }

        draw_text_layout(target, &layout, rect.left, rect.top, normal);
    }
}

// ---------- INPUT BAR ----------

// pub fn input_bar(
//     target: &ID2D1HwndRenderTarget,
//     dwrite: &IDWriteFactory,
//     rect: D2D_RECT_F,
//     text: &str,
//     cursor: usize,
//     format: &IDWriteTextFormat,
//     bg: &ID2D1SolidColorBrush,
//     border: &ID2D1SolidColorBrush,
//     text_brush: &ID2D1SolidColorBrush,
//     cursor_brush: &ID2D1SolidColorBrush,
// ) {
//     let rounded = D2D1_ROUNDED_RECT {
//         rect,
//         radiusX: 6.0,
//         radiusY: 6.0,
//     };

//     unsafe {
//         target.FillRoundedRectangle(&rounded, bg);
//         target.DrawRoundedRectangle(&rounded, border, 0.8, None);
//     }

//     let text_rect = D2D_RECT_F {
//         left: rect.left + 10.0,
//         top: rect.top + 6.0,
//         right: rect.right,
//         bottom: rect.bottom,
//     };

//     // Draw text
//     text(target, text, format, text_rect, text_brush);

//     // Cursor (simple, replace later with HitTest)
//     let sub = &text[..cursor.min(text.len())];
//     let utf16: Vec<u16> = sub.encode_utf16().collect();

//     unsafe {
//         if let Ok(layout) = dwrite.CreateTextLayout(&utf16, format, 1000.0, 100.0) {
//             let mut metrics = DWRITE_TEXT_METRICS::default();
//             let _ = layout.GetMetrics(&mut metrics);

//             let x = text_rect.left + metrics.width;

//             let cursor_rect = D2D_RECT_F {
//                 left: x,
//                 top: rect.top + 6.0,
//                 right: x + 2.0,
//                 bottom: rect.bottom - 6.0,
//             };

//             target.FillRectangle(&cursor_rect, cursor_brush);
//         }
//     }
// }
