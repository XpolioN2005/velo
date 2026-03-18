use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

pub const TITLE_BAR_H: f32 = 26.0;
pub const QUERY_BAR_H: f32 = 48.0;
pub const ROW_H: f32 = 40.0;
pub const DIVIDER_H: f32 = 1.0;
pub const PADDING_H: f32 = 16.0;
pub const MAX_ROWS: usize = 8;

pub fn title_bar_rect(w: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: 0.0,
        top: 0.0,
        right: w,
        bottom: TITLE_BAR_H,
    }
}

pub fn query_bar_rect(w: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: PADDING_H,
        top: TITLE_BAR_H,
        right: w - PADDING_H,
        bottom: TITLE_BAR_H + QUERY_BAR_H,
    }
}

pub fn border_rect(w: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: 1.0,
        top: TITLE_BAR_H + 1.0,
        right: w - 1.0,
        bottom: TITLE_BAR_H + QUERY_BAR_H - 1.0,
    }
}

pub fn row_rect(w: f32, index: usize) -> D2D_RECT_F {
    let y = TITLE_BAR_H + QUERY_BAR_H + (index as f32 * ROW_H);
    D2D_RECT_F {
        left: 0.0,
        top: y,
        right: w,
        bottom: y + ROW_H,
    }
}

pub fn name_rect(row: D2D_RECT_F) -> D2D_RECT_F {
    D2D_RECT_F {
        left: row.left + PADDING_H,
        top: row.top,
        right: row.right * 0.6,
        bottom: row.bottom,
    }
}

pub fn desc_rect(row: D2D_RECT_F) -> D2D_RECT_F {
    D2D_RECT_F {
        left: row.right * 0.6,
        top: row.top,
        right: row.right - PADDING_H,
        bottom: row.bottom,
    }
}

pub fn divider_rect(w: f32, after_index: usize) -> D2D_RECT_F {
    let y = TITLE_BAR_H + QUERY_BAR_H + (after_index as f32 * ROW_H);
    D2D_RECT_F {
        left: PADDING_H,
        top: y,
        right: w - PADDING_H,
        bottom: y + DIVIDER_H,
    }
}

pub fn window_height(result_count: usize) -> i32 {
    let rows = result_count.min(MAX_ROWS);
    (TITLE_BAR_H + QUERY_BAR_H + rows as f32 * ROW_H) as i32
}
