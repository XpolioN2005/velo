use windows::{Win32::Foundation::RECT, Win32::UI::WindowsAndMessaging::GetClientRect};

use crate::app::{AppState, InputMode};
use crate::renderer::{Renderer, draw, layout, theme};

pub fn draw_palette(
    renderer: &Renderer,
    app: &AppState,
    hwnd: windows::Win32::Foundation::HWND,
    caret_visible: bool,
) {
    unsafe {
        renderer.begin();
        renderer.clear();

        // Get window width
        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let w = rc.right as f32;

        // --- Title bar ---
        let title_rect = layout::title_bar_rect(w);
        let _ = draw::draw_rect_filled(&renderer.target, title_rect, theme::TITLE_BAR_BG);

        let title_text_rect = windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F {
            left: title_rect.left + layout::PADDING_H,
            top: title_rect.top,
            right: title_rect.right,
            bottom: title_rect.bottom,
        };
        let _ = draw::draw_text(
            &renderer.target,
            "velo",
            &renderer.text_ui,
            title_text_rect,
            theme::TITLE_TEXT,
        );

        // Title bar bottom border
        let _ = draw::draw_rect_filled(
            &renderer.target,
            windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F {
                left: 0.0,
                top: layout::TITLE_BAR_H - 1.0,
                right: w,
                bottom: layout::TITLE_BAR_H,
            },
            theme::DIVIDER,
        );

        // Outer border
        let border_color = if app.focused {
            theme::BORDER_FOCUS
        } else {
            theme::BORDER_UNFOCUS
        };
        let _ =
            draw::draw_rect_outline(&renderer.target, layout::border_rect(w), border_color, 1.5);

        // --- Query / Arg Input Bar ---
        let text_rect = layout::query_bar_rect(w);
        let active_buf = app.active_buf();
        let show_placeholder = active_buf.is_empty();

        match &app.mode {
            InputMode::Query => {
                if show_placeholder {
                    let _ = draw::draw_text(
                        &renderer.target,
                        "Search...",
                        &renderer.text_ui,
                        text_rect,
                        theme::TEXT_DIM,
                    );
                } else {
                    if let Some(sel) = app.selection {
                        let _ = draw::draw_selection_highlight(
                            &renderer.target,
                            &renderer.dwrite,
                            &app.query,
                            &renderer.text_ui,
                            text_rect,
                            sel,
                            theme::SELECTION_BG,
                        );
                    }
                    if caret_visible && app.selection.is_none() {
                        let _ = draw::draw_caret(
                            &renderer.target,
                            &renderer.dwrite,
                            &app.query,
                            &renderer.text_ui,
                            text_rect,
                            app.cursor,
                            renderer.scale,
                            theme::TEXT,
                        );
                    }
                    let _ = draw::draw_text(
                        &renderer.target,
                        &app.query,
                        &renderer.text_ui,
                        text_rect,
                        theme::TEXT,
                    );
                }
            }
            InputMode::ArgInput { .. } => {
                let prompt = app.current_prompt().unwrap_or("Enter value:");
                if show_placeholder {
                    let _ = draw::draw_text(
                        &renderer.target,
                        prompt,
                        &renderer.text_ui,
                        text_rect,
                        theme::TEXT_DIM,
                    );
                } else {
                    if let Some(sel) = app.selection {
                        let _ = draw::draw_selection_highlight(
                            &renderer.target,
                            &renderer.dwrite,
                            &app.arg_buffer,
                            &renderer.text_ui,
                            text_rect,
                            sel,
                            theme::SELECTION_BG,
                        );
                    }
                    if caret_visible && app.selection.is_none() {
                        let _ = draw::draw_caret(
                            &renderer.target,
                            &renderer.dwrite,
                            &app.arg_buffer,
                            &renderer.text_ui,
                            text_rect,
                            app.cursor,
                            renderer.scale,
                            theme::TEXT,
                        );
                    }
                    let _ = draw::draw_text(
                        &renderer.target,
                        &app.arg_buffer,
                        &renderer.text_ui,
                        text_rect,
                        theme::TEXT,
                    );
                }
            }
        }

        // If no results or in ArgInput mode, finish early
        if app.results.is_empty() || matches!(app.mode, InputMode::ArgInput { .. }) {
            renderer.end().unwrap();
            return;
        }

        // --- Results List ---

        for (i, matched) in app.results.iter().enumerate() {
            let row = layout::row_rect(w, i);

            // Row highlight
            if i == app.selected {
                let _ = draw::draw_rect_filled(&renderer.target, row, theme::HIGHLIGHT_BG);
            }

            // Draw name with highlights
            let _ = draw::draw_text_highlighted(
                &renderer.target,
                &renderer.dwrite,
                &matched.name,
                &renderer.text_ui,
                layout::name_rect(row),
                theme::TEXT,
                theme::ACCENT,
                &matched.match_indices,
            );

            // Draw description
            let _ = draw::draw_text(
                &renderer.target,
                &matched.description,
                &renderer.text_desc,
                layout::desc_rect(row),
                theme::TEXT_DESC,
            );
        }

        renderer.end().unwrap();
    }
}
