// ── crate imports ─────────────────────────
use crate::app::InputMode;
use crate::renderer::{Renderer, layout};
use crate::window::state::WindowState;

// ── Windows API imports ───────────────────
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::InvalidateRect,
    UI::Input::KeyboardAndMouse::SetFocus,
    UI::WindowsAndMessaging::{
        SW_HIDE, SW_SHOW, SWP_NOMOVE, SWP_NOZORDER, SetForegroundWindow, SetWindowPos, ShowWindow,
    },
};
pub unsafe fn hide(hwnd: HWND, ptr: *mut WindowState) {
    unsafe {
        (*ptr).app.clear_query();
        let h = layout::window_height(0);
        let _ = SetWindowPos(hwnd, None, 0, 0, (*ptr).win_w, h, SWP_NOMOVE | SWP_NOZORDER);
        (*ptr).renderer = None;
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub unsafe fn show(hwnd: HWND, ptr: *mut WindowState) {
    unsafe {
        if (*ptr).renderer.is_none() {
            (*ptr).renderer = Renderer::new(hwnd).ok();
        }
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(hwnd));
        (*ptr).caret_visible = true;
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

pub unsafe fn resize_to_results(hwnd: HWND, state: &WindowState) {
    unsafe {
        let count = match &state.app.mode {
            InputMode::ArgInput { .. } => 0,
            InputMode::Query => state.app.results.len(),
        };
        let h = layout::window_height(count);
        let _ = SetWindowPos(hwnd, None, 0, 0, state.win_w, h, SWP_NOMOVE | SWP_NOZORDER);
    }
}
