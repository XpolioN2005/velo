use windows::{Win32::Foundation::RECT, Win32::UI::WindowsAndMessaging::GetClientRect};

use crate::app::{AppState, InputMode};
use crate::command::CommandRef;
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

        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let w = rc.right as f32;

        // title bar
        let _ = draw::draw_rect_filled(
            &renderer.target,
            layout::title_bar_rect(w),
            theme::TITLE_BAR_BG,
        );
        let title_rect = layout::title_bar_rect(w);
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

        // title bar bottom border
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

        // outer border
        let border_color = if app.focused {
            theme::BORDER_FOCUS
        } else {
            theme::BORDER_UNFOCUS
        };
        let _ =
            draw::draw_rect_outline(&renderer.target, layout::border_rect(w), border_color, 1.5);

        // query bar
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
                    // selection highlight
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
                    // caret — only when no selection
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
                    // selection highlight
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
                    // caret — only when no selection
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
                        &app.arg_buffer,
                        &renderer.text_ui,
                        text_rect,
                        theme::TEXT,
                    );
                }
            }
        }

        if app.results.is_empty() || matches!(app.mode, InputMode::ArgInput { .. }) {
            renderer.end().unwrap();
            return;
        }

        let mut last_was_builtin = matches!(
            app.results.first().map(|m| m.cmd_ref),
            Some(CommandRef::BuiltIn(_))
        );

        for (i, matched) in app.results.iter().enumerate() {
            let cmd_ref = matched.cmd_ref;
            let is_builtin = matches!(cmd_ref, CommandRef::BuiltIn(_));

            if !is_builtin && last_was_builtin && i > 0 {
                let _ = draw::draw_rect_filled(
                    &renderer.target,
                    layout::divider_rect(w, i),
                    theme::DIVIDER,
                );
            }
            last_was_builtin = is_builtin;

            let row = layout::row_rect(w, i);

            if i == app.selected {
                let _ = draw::draw_rect_filled(&renderer.target, row, theme::HIGHLIGHT_BG);
            }

            let (name, desc) = match cmd_ref {
                CommandRef::BuiltIn(idx) => {
                    let cmd = &crate::command::BUILT_INS[idx];
                    (cmd.name, cmd.description)
                }
                CommandRef::User(idx) => {
                    let cmd = &app.user_commands[idx];
                    (cmd.name.as_str(), cmd.description.as_str())
                }
            };

            let _ = draw::draw_text_highlighted(
                &renderer.target,
                &renderer.dwrite,
                name,
                &renderer.text_ui,
                layout::name_rect(row),
                theme::TEXT,
                theme::ACCENT,
                &matched.match_indices,
            );
            let _ = draw::draw_text(
                &renderer.target,
                desc,
                &renderer.text_desc,
                layout::desc_rect(row),
                theme::TEXT_DESC,
            );
        }

        renderer.end().unwrap();
    }
}
