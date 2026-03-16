use windows::{
    Win32::Foundation::RECT, Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    Win32::UI::WindowsAndMessaging::GetClientRect, core::*,
};

use crate::app::AppState;
use crate::renderer::{Renderer, draw, theme};

const SEARCH_PADDING: f32 = 16.0;

pub fn draw_palette(renderer: &Renderer, app: &AppState, hwnd: windows::Win32::Foundation::HWND) {
    unsafe {
        renderer.begin();
        renderer.clear();

        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let w = rc.right as f32;
        let h = rc.bottom as f32;

        // Border — changes color based on focus
        let border_color = if app.focused {
            theme::BORDER_FOCUS
        } else {
            theme::BORDER_UNFOCUS
        };

        let _ = draw::draw_rect_outline(
            &renderer.target,
            D2D_RECT_F {
                left: 1.0,
                top: 1.0,
                right: w - 1.0,
                bottom: h - 1.0,
            },
            border_color,
            1.5,
        );

        // Query text or placeholder
        let text_rect = D2D_RECT_F {
            left: SEARCH_PADDING,
            top: 0.0,
            right: w - SEARCH_PADDING,
            bottom: h,
        };

        if app.query.is_empty() {
            let _ = draw::draw_text(
                &renderer.target,
                "Search...",
                &renderer.text_ui,
                text_rect,
                theme::TEXT_DIM,
            );
        } else {
            let _ = draw::draw_text(
                &renderer.target,
                &app.query,
                &renderer.text_ui,
                text_rect,
                theme::TEXT,
            );
        }

        renderer.end().unwrap();
    }
}
