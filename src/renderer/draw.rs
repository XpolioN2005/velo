use windows::{
    Win32::Graphics::{
        Direct2D::{
            Common::{D2D_RECT_F, D2D1_COLOR_F},
            ID2D1HwndRenderTarget, ID2D1SolidColorBrush,
        },
        DirectWrite::{
            DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_REGULAR,
            DWRITE_HIT_TEST_METRICS, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
            DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TRIMMING,
            DWRITE_TRIMMING_GRANULARITY_CHARACTER, IDWriteFactory, IDWriteTextFormat,
            IDWriteTextLayout,
        },
    },
    core::*,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Point2F {
    x: f32,
    y: f32,
}

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
            format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING)?;
            format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
            let trimming_sign = dwrite.CreateEllipsisTrimmingSign(&format)?;
            let trimming = DWRITE_TRIMMING {
                granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                delimiter: 0,
                delimiterCount: 0,
            };
            format.SetTrimming(&trimming, &trimming_sign)?;
            Ok(Self { format })
        }
    }

    pub fn new_right(dwrite: &IDWriteFactory, size: f32) -> Result<Self> {
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
            format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING)?;
            format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
            let trimming_sign = dwrite.CreateEllipsisTrimmingSign(&format)?;
            let trimming = DWRITE_TRIMMING {
                granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                delimiter: 0,
                delimiterCount: 0,
            };
            format.SetTrimming(&trimming, &trimming_sign)?;
            Ok(Self { format })
        }
    }
}

pub fn draw_text(
    target: &ID2D1HwndRenderTarget,
    text: &str,
    fmt: &TextFormat,
    rect: D2D_RECT_F,
    color: D2D1_COLOR_F,
) -> Result<()> {
    unsafe {
        let brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&color, None)?;
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

pub fn draw_text_highlighted(
    target: &ID2D1HwndRenderTarget,
    dwrite: &IDWriteFactory,
    text: &str,
    fmt: &TextFormat,
    rect: D2D_RECT_F,
    text_color: D2D1_COLOR_F,
    highlight_color: D2D1_COLOR_F,
    match_indices: &[usize],
) -> Result<()> {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let rect_w = rect.right - rect.left;
        let rect_h = rect.bottom - rect.top;

        let layout: IDWriteTextLayout =
            dwrite.CreateTextLayout(&wide, &fmt.format, rect_w, rect_h)?;

        let text_brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&text_color, None)?;
        target.DrawTextLayout(
            std::mem::transmute(Point2F {
                x: rect.left,
                y: rect.top,
            }),
            &layout,
            &text_brush,
            Default::default(),
        );

        if match_indices.is_empty() {
            return Ok(());
        }

        let ranges = consecutive_ranges(match_indices);
        let highlight_brush: ID2D1SolidColorBrush =
            target.CreateSolidColorBrush(&highlight_color, None)?;

        for (start, len) in ranges {
            let mut actual_count = 0u32;
            let _ = layout.HitTestTextRange(
                start as u32,
                len as u32,
                rect.left,
                rect.top,
                None,
                &mut actual_count,
            );

            if actual_count == 0 {
                continue;
            }

            let mut hit_metrics = vec![DWRITE_HIT_TEST_METRICS::default(); actual_count as usize];
            let _ = layout.HitTestTextRange(
                start as u32,
                len as u32,
                rect.left,
                rect.top,
                Some(hit_metrics.as_mut_slice()),
                &mut actual_count,
            );

            for m in &hit_metrics[..actual_count as usize] {
                let char_rect = D2D_RECT_F {
                    left: m.left,
                    top: rect.top,
                    right: m.left + m.width,
                    bottom: rect.bottom,
                };
                target.PushAxisAlignedClip(&char_rect, Default::default());
                target.DrawTextLayout(
                    std::mem::transmute(Point2F {
                        x: rect.left,
                        y: rect.top,
                    }),
                    &layout,
                    &highlight_brush,
                    Default::default(),
                );
                target.PopAxisAlignedClip();
            }
        }

        Ok(())
    }
}

fn consecutive_ranges(indices: &[usize]) -> Vec<(usize, usize)> {
    if indices.is_empty() {
        return vec![];
    }
    let mut ranges = Vec::new();
    let mut start = indices[0];
    let mut len = 1;
    for &idx in &indices[1..] {
        if idx == start + len {
            len += 1;
        } else {
            ranges.push((start, len));
            start = idx;
            len = 1;
        }
    }
    ranges.push((start, len));
    ranges
}

pub fn draw_rect_outline(
    target: &ID2D1HwndRenderTarget,
    rect: D2D_RECT_F,
    color: D2D1_COLOR_F,
    width: f32,
) -> Result<()> {
    unsafe {
        let brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&color, None)?;
        target.DrawRectangle(&rect, &brush, width, None);
        Ok(())
    }
}

pub fn draw_rect_filled(
    target: &ID2D1HwndRenderTarget,
    rect: D2D_RECT_F,
    color: D2D1_COLOR_F,
) -> Result<()> {
    unsafe {
        let brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&color, None)?;
        target.FillRectangle(&rect, &brush);
        Ok(())
    }
}

pub fn draw_selection_highlight(
    target: &ID2D1HwndRenderTarget,
    dwrite: &IDWriteFactory,
    text: &str,
    fmt: &TextFormat,
    rect: D2D_RECT_F,
    selection: (usize, usize), // byte indices
    color: D2D1_COLOR_F,
) -> Result<()> {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let rect_w = rect.right - rect.left;
        let rect_h = rect.bottom - rect.top;
        let layout: IDWriteTextLayout =
            dwrite.CreateTextLayout(&wide, &fmt.format, rect_w, rect_h)?;

        // convert byte indices to utf-16 code unit indices
        let start_cu = byte_to_utf16_offset(text, selection.0);
        let end_cu = byte_to_utf16_offset(text, selection.1);

        if start_cu >= end_cu {
            return Ok(());
        }

        let mut actual_count = 0u32;
        let _ = layout.HitTestTextRange(
            start_cu as u32,
            (end_cu - start_cu) as u32,
            rect.left,
            rect.top,
            None,
            &mut actual_count,
        );

        if actual_count == 0 {
            return Ok(());
        }

        let mut hit_metrics = vec![DWRITE_HIT_TEST_METRICS::default(); actual_count as usize];
        let _ = layout.HitTestTextRange(
            start_cu as u32,
            (end_cu - start_cu) as u32,
            rect.left,
            rect.top,
            Some(hit_metrics.as_mut_slice()),
            &mut actual_count,
        );

        let brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&color, None)?;
        for m in &hit_metrics[..actual_count as usize] {
            let sel_rect = D2D_RECT_F {
                left: m.left,
                top: rect.top + 10.0, // small vertical inset so it doesn't bleed into border
                right: m.left + m.width,
                bottom: rect.bottom - 10.0,
            };
            target.FillRectangle(&sel_rect, &brush);
        }

        Ok(())
    }
}

fn byte_to_utf16_offset(s: &str, byte_pos: usize) -> usize {
    s[..byte_pos.min(s.len())].encode_utf16().count()
}

pub fn draw_caret(
    target: &ID2D1HwndRenderTarget,
    dwrite: &IDWriteFactory,
    text: &str,
    fmt: &TextFormat,
    rect: D2D_RECT_F,
    cursor: usize,
    scale: f32,
    color: D2D1_COLOR_F,
) -> Result<()> {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let rect_w = rect.right - rect.left;
        let rect_h = rect.bottom - rect.top;
        let layout: IDWriteTextLayout =
            dwrite.CreateTextLayout(&wide, &fmt.format, rect_w, rect_h)?;

        let cursor_cu = byte_to_utf16_offset(text, cursor);

        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut metrics = DWRITE_HIT_TEST_METRICS::default();
        let _ = layout.HitTestTextPosition(cursor_cu as u32, false, &mut x, &mut y, &mut metrics);

        let caret_x = rect.left + x;
        let caret_w = 1.5 / scale;
        let inset = 15.0 / scale;

        let caret_rect = D2D_RECT_F {
            left: caret_x,
            top: rect.top + inset,
            right: caret_x + caret_w,
            bottom: rect.bottom - inset,
        };

        let brush: ID2D1SolidColorBrush = target.CreateSolidColorBrush(&color, None)?;
        target.FillRectangle(&caret_rect, &brush);

        Ok(())
    }
}
