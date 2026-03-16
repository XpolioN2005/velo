use windows::{Win32::Foundation::RECT, Win32::UI::WindowsAndMessaging::GetClientRect};

use crate::app::AppState;
use crate::command::CommandRef;
use crate::renderer::{Renderer, draw, layout, theme};

pub fn draw_palette(renderer: &Renderer, app: &AppState, hwnd: windows::Win32::Foundation::HWND) {
    unsafe {
        renderer.begin();
        renderer.clear();

        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let w = rc.right as f32;

        let border_color = if app.focused {
            theme::BORDER_FOCUS
        } else {
            theme::BORDER_UNFOCUS
        };
        let _ =
            draw::draw_rect_outline(&renderer.target, layout::border_rect(w), border_color, 1.5);

        let text_rect = layout::query_bar_rect(w);
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

        if app.results.is_empty() {
            renderer.end().unwrap();
            return;
        }

        let mut last_was_builtin = matches!(app.results.first(), Some(CommandRef::BuiltIn(_)));

        for (i, cmd_ref) in app.results.iter().enumerate() {
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
                    let cmd = &crate::command::BUILT_INS[*idx];
                    (cmd.name, cmd.description)
                }
                CommandRef::User(idx) => {
                    let cmd = &app.user_commands[*idx];
                    (cmd.name.as_str(), cmd.description.as_str())
                }
            };

            let _ = draw::draw_text(
                &renderer.target,
                name,
                &renderer.text_ui,
                layout::name_rect(row),
                theme::TEXT,
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
