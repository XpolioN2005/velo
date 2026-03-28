// ── crate imports ─────────────────────────
use crate::app::WindowAction;
use crate::renderer::{layout, palette};
use crate::window::{
    clipboard::{clipboard_read, clipboard_write},
    input::ctrl_held,
    state::*,
    window::{hide, resize_to_results, show},
};

// ── Windows API imports ───────────────────
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GWLP_USERDATA, GetWindowLongPtrW, GetWindowRect, HTCAPTION, KillTimer,
    PostQuitMessage, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_EXITSIZEMOVE, WM_HOTKEY, WM_KEYDOWN,
    WM_KILLFOCUS, WM_NCHITTEST, WM_PAINT, WM_SETFOCUS, WM_SIZE, WM_TIMER,
};

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;

        match msg {
            WM_NCHITTEST => {
                let base = DefWindowProcW(hwnd, msg, wparam, lparam);
                if !ptr.is_null() {
                    let screen_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                    let mut wr = RECT::default();
                    let _ = GetWindowRect(hwnd, &mut wr);
                    let client_y = screen_y - wr.top;
                    let scale = (*ptr).renderer.as_ref().map(|r| r.scale).unwrap_or(1.0);
                    let title_bar_h = (layout::TITLE_BAR_H * scale) as i32;
                    if client_y >= 0 && client_y < title_bar_h {
                        return LRESULT(HTCAPTION as isize);
                    }
                }
                base
            }
            WM_EXITSIZEMOVE => {
                if !ptr.is_null() {
                    let mut wr = RECT::default();
                    let _ = GetWindowRect(hwnd, &mut wr);
                    (*ptr).app.save_position(wr.left, wr.top);
                }
                LRESULT(0)
            }
            WM_HOTKEY => {
                if wparam.0 as i32 == HOTKEY_ID && !ptr.is_null() {
                    show(hwnd, ptr);
                }
                LRESULT(0)
            }
            WM_TIMER => {
                if !ptr.is_null() && wparam.0 == CARET_TIMER_ID {
                    (*ptr).caret_visible = !(*ptr).caret_visible;
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }

            WM_SIZE => {
                if !ptr.is_null() {
                    if let Some(r) = &(*ptr).renderer {
                        let w = (lparam.0 & 0xFFFF) as u32;
                        let h = ((lparam.0 >> 16) & 0xFFFF) as u32;
                        if w > 0 && h > 0 {
                            let _ = r.resize(w, h);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_SETFOCUS => {
                if !ptr.is_null() {
                    (*ptr).app.focused = true;
                    (*ptr).caret_visible = true;
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                if !ptr.is_null() {
                    (*ptr).app.focused = false;
                    hide(hwnd, ptr);
                }
                LRESULT(0)
            }
            WM_CHAR => {
                if !ptr.is_null() {
                    if ctrl_held() {
                        return LRESULT(0);
                    }
                    if let Some(c) = char::from_u32(wparam.0 as u32) {
                        if !c.is_control() {
                            (*ptr).app.push_char(c);
                            (*ptr).caret_visible = true;
                            resize_to_results(hwnd, &*ptr);
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                if !ptr.is_null() {
                    let ctrl = ctrl_held();
                    let vk = wparam.0 as u32;

                    if ctrl {
                        match vk {
                            VK_A => {
                                (*ptr).app.select_all();
                                let _ = InvalidateRect(Some(hwnd), None, false);
                            }
                            VK_C => {
                                if let Some(text) = (*ptr).app.copy_text() {
                                    clipboard_write(hwnd, &text);
                                }
                            }
                            VK_X => {
                                if let Some(text) = (*ptr).app.cut_text() {
                                    clipboard_write(hwnd, &text);
                                    (*ptr).caret_visible = true;
                                    resize_to_results(hwnd, &*ptr);
                                    let _ = InvalidateRect(Some(hwnd), None, false);
                                }
                            }
                            VK_V => {
                                if let Some(text) = clipboard_read() {
                                    (*ptr).app.paste_text(&text);
                                    (*ptr).caret_visible = true;
                                    resize_to_results(hwnd, &*ptr);
                                    let _ = InvalidateRect(Some(hwnd), None, false);
                                }
                            }
                            _ => {}
                        }
                        return LRESULT(0);
                    }

                    match vk {
                        VK_BACK => {
                            (*ptr).app.pop_char();
                            (*ptr).caret_visible = true;
                            resize_to_results(hwnd, &*ptr);
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_ESCAPE => {
                            if (*ptr).app.escape_should_hide() {
                                hide(hwnd, ptr);
                            } else {
                                (*ptr).app.escape();
                                (*ptr).caret_visible = true;
                                resize_to_results(hwnd, &*ptr);
                                let _ = InvalidateRect(Some(hwnd), None, false);
                            }
                        }
                        VK_LEFT => {
                            (*ptr).app.move_cursor_left();
                            (*ptr).caret_visible = true;
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_RIGHT => {
                            (*ptr).app.move_cursor_right();
                            (*ptr).caret_visible = true;
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_UP => {
                            (*ptr).app.select_prev();
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_DOWN => {
                            (*ptr).app.select_next();
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_RETURN => match (*ptr).app.enter(hwnd.0 as isize) {
                            WindowAction::Quit => {
                                PostQuitMessage(0);
                            }
                            WindowAction::Hide => {
                                hide(hwnd, ptr);
                            }
                            WindowAction::Nothing => {
                                (*ptr).caret_visible = true;
                                resize_to_results(hwnd, &*ptr);
                                let _ = InvalidateRect(Some(hwnd), None, false);
                            }
                            WindowAction::RunSequence => {}
                        },
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_PAINT => {
                if !ptr.is_null() {
                    if let Some(r) = &(*ptr).renderer {
                        palette::draw_palette(r, &(*ptr).app, hwnd, (*ptr).caret_visible);
                    }
                }
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_CLOSE => {
                if !ptr.is_null() {
                    hide(hwnd, ptr);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                if !ptr.is_null() {
                    let _ = KillTimer(Some(hwnd), CARET_TIMER_ID);
                    drop(Box::from_raw(ptr));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
