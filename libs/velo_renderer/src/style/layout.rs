use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

// You can later plug DPI scaling here
const PADDING: f32 = 10.0;
const ITEM_HEIGHT: f32 = 32.0;
const SPACING: f32 = 4.0;
const INPUT_HEIGHT: f32 = 40.0;

// ---------- INPUT ----------

pub fn input_rect(width: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: PADDING,
        top: PADDING,
        right: width - PADDING,
        bottom: PADDING + INPUT_HEIGHT,
    }
}

// ---------- ROW ----------

pub fn row_rect(width: f32, index: usize) -> D2D_RECT_F {
    let top = PADDING + INPUT_HEIGHT + SPACING + index as f32 * (ITEM_HEIGHT + SPACING);

    D2D_RECT_F {
        left: PADDING,
        top,
        right: width - PADDING,
        bottom: top + ITEM_HEIGHT,
    }
}

// ---------- TEXT AREAS ----------

pub fn name_rect(row: D2D_RECT_F) -> D2D_RECT_F {
    D2D_RECT_F {
        left: row.left + 10.0,
        top: row.top + 4.0,
        right: row.right,
        bottom: row.bottom,
    }
}

pub fn desc_rect(row: D2D_RECT_F) -> D2D_RECT_F {
    D2D_RECT_F {
        left: row.left + 10.0,
        top: row.top + 18.0,
        right: row.right,
        bottom: row.bottom,
    }
}
