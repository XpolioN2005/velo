use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

pub const QUERY_BAR_H: f32 = 48.0;
pub const ROW_H: f32 = 56.0;
pub const DIVIDER_H: f32 = 1.0;
pub const PADDING_H: f32 = 16.0;
pub const MAX_ROWS: usize = 8;

pub fn query_bar_rect(w: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: 0.0,
        top: 0.0,
        right: w,
        bottom: QUERY_BAR_H,
    }
}

pub fn row_rect(w: f32, index: usize) -> D2D_RECT_F {
    let y = QUERY_BAR_H + (index as f32 * ROW_H);
    D2D_RECT_F {
        left: 0.0,
        top: y,
        right: w,
        bottom: y + ROW_H,
    }
}

// Name sits in top half of row, description in bottom half
pub fn name_rect(row: D2D_RECT_F) -> D2D_RECT_F {
    D2D_RECT_F {
        left: row.left + PADDING_H,
        top: row.top + 8.0,
        right: row.right - PADDING_H,
        bottom: row.top + ROW_H * 0.52,
    }
}

pub fn desc_rect(row: D2D_RECT_F) -> D2D_RECT_F {
    D2D_RECT_F {
        left: row.left + PADDING_H,
        top: row.top + ROW_H * 0.52,
        right: row.right - PADDING_H,
        bottom: row.bottom - 8.0,
    }
}

pub fn divider_rect(w: f32, after_index: usize) -> D2D_RECT_F {
    let y = QUERY_BAR_H + (after_index as f32 * ROW_H);
    D2D_RECT_F {
        left: PADDING_H,
        top: y,
        right: w - PADDING_H,
        bottom: y + DIVIDER_H,
    }
}

pub fn window_height(result_count: usize) -> i32 {
    if result_count == 0 {
        return QUERY_BAR_H as i32;
    }
    let rows = result_count.min(MAX_ROWS);
    (QUERY_BAR_H + rows as f32 * ROW_H) as i32
}
