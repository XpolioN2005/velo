use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, UpdateWindow},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
            },
            LibraryLoader::GetModuleHandleW,
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyState, MOD_ALT, MOD_CONTROL, RegisterHotKey, SetFocus, VK_CONTROL, VK_P,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
                GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, HTCAPTION, IDC_ARROW,
                LoadCursorW, MSG, PostQuitMessage, RegisterClassExW, SM_CXSCREEN, SM_CYSCREEN,
                SW_HIDE, SW_SHOW, SWP_NOMOVE, SWP_NOZORDER, SetForegroundWindow, SetWindowLongPtrW,
                SetWindowPos, ShowWindow, TranslateMessage, WM_CHAR, WM_CLOSE, WM_DESTROY,
                WM_EXITSIZEMOVE, WM_HOTKEY, WM_KEYDOWN, WM_KILLFOCUS, WM_NCHITTEST, WM_PAINT,
                WM_SETFOCUS, WM_SIZE, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
                WS_VISIBLE,
            },
        },
    },
    core::*,
};

use crate::app::{AppState, InputMode};
use crate::command::WindowAction;
use crate::renderer::{Renderer, layout, palette};

const WIN_WIDTH_RATIO: f32 = 0.40;
const WIN_TOP_RATIO: f32 = 0.08;

const VK_BACK: u32 = 0x08;
const VK_ESCAPE: u32 = 0x1B;
const VK_RETURN: u32 = 0x0D;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;
const VK_LEFT: u32 = 0x25;
const VK_RIGHT: u32 = 0x27;
const VK_A: u32 = 0x41;
const VK_C: u32 = 0x43;
const VK_V: u32 = 0x56;
const VK_X: u32 = 0x58;

const HOTKEY_ID: i32 = 1;

struct WindowState {
    renderer: Option<Renderer>,
    app: AppState,
    win_w: i32,
}

unsafe fn hide(hwnd: HWND, ptr: *mut WindowState) {
    unsafe {
        (*ptr).app.clear_query();
        let h = layout::window_height(0);
        let _ = SetWindowPos(hwnd, None, 0, 0, (*ptr).win_w, h, SWP_NOMOVE | SWP_NOZORDER);
        (*ptr).renderer = None;
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

unsafe fn show(hwnd: HWND, ptr: *mut WindowState) {
    unsafe {
        if (*ptr).renderer.is_none() {
            (*ptr).renderer = Renderer::new(hwnd).ok();
        }
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(hwnd));
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

unsafe fn resize_to_results(hwnd: HWND, state: &WindowState) {
    unsafe {
        let count = match &state.app.mode {
            InputMode::ArgInput { .. } => 0,
            InputMode::Query => state.app.results.len(),
        };
        let h = layout::window_height(count);
        let _ = SetWindowPos(hwnd, None, 0, 0, state.win_w, h, SWP_NOMOVE | SWP_NOZORDER);
    }
}

fn ctrl_held() -> bool {
    unsafe { GetKeyState(VK_CONTROL.0 as i32) & (0x8000u16 as i16) != 0 }
}

// ── clipboard ─────────────────────────────────────────────────────────────────

unsafe fn clipboard_write(hwnd: HWND, text: &str) {
    unsafe {
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = utf16.len() * 2;

        let hmem = match GlobalAlloc(GMEM_MOVEABLE, byte_len) {
            Ok(h) => h,
            Err(_) => return,
        };

        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            return;
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u16, utf16.len());
        let _ = GlobalUnlock(hmem);

        if OpenClipboard(Some(hwnd)).is_ok() {
            let _ = EmptyClipboard();
            let _ = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(hmem.0)));
            let _ = CloseClipboard();
        }
    }
}

unsafe fn clipboard_read() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let handle = match GetClipboardData(CF_UNICODETEXT.0 as u32) {
            Ok(h) => h,
            Err(_) => {
                let _ = CloseClipboard();
                return None;
            }
        };
        let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u16;
        if ptr.is_null() {
            let _ = CloseClipboard();
            return None;
        }

        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        let text = String::from_utf16_lossy(slice).to_string();

        let _ = GlobalUnlock(HGLOBAL(handle.0));
        let _ = CloseClipboard();
        Some(text)
    }
}

// ── window setup ──────────────────────────────────────────────────────────────

pub fn create_and_run() {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
        let class_name = w!("velo_palette");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let win_w = (screen_w as f32 * WIN_WIDTH_RATIO) as i32;
        let win_h = layout::window_height(0);

        let app = AppState::new();
        let win_x = app.config.window_x.unwrap_or((screen_w - win_w) / 2);
        let win_y = app
            .config
            .window_y
            .unwrap_or((screen_h as f32 * WIN_TOP_RATIO) as i32);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!("velo"),
            WS_POPUP | WS_VISIBLE,
            win_x,
            win_y,
            win_w,
            win_h,
            None,
            None,
            Some(hinstance),
            None,
        )
        .unwrap();

        let state = Box::new(WindowState {
            renderer: Renderer::new(hwnd).ok(),
            app,
            win_w,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        let _ = UpdateWindow(hwnd);
        let _ = RegisterHotKey(Some(hwnd), HOTKEY_ID, MOD_CONTROL | MOD_ALT, VK_P.0 as u32);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// ── wnd_proc ──────────────────────────────────────────────────────────────────

unsafe extern "system" fn wnd_proc(
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
                    // suppress Ctrl+key chars (they arrive as control chars 1–26)
                    if ctrl_held() {
                        return LRESULT(0);
                    }
                    if let Some(c) = char::from_u32(wparam.0 as u32) {
                        if !c.is_control() {
                            (*ptr).app.push_char(c);
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

                    // ── Ctrl combos ───────────────────────────────────────────
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
                                    resize_to_results(hwnd, &*ptr);
                                    let _ = InvalidateRect(Some(hwnd), None, false);
                                }
                            }
                            VK_V => {
                                if let Some(text) = clipboard_read() {
                                    (*ptr).app.paste_text(&text);
                                    resize_to_results(hwnd, &*ptr);
                                    let _ = InvalidateRect(Some(hwnd), None, false);
                                }
                            }
                            _ => {}
                        }
                        return LRESULT(0);
                    }

                    // ── non-Ctrl keys ─────────────────────────────────────────
                    match vk {
                        VK_BACK => {
                            (*ptr).app.pop_char();
                            resize_to_results(hwnd, &*ptr);
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_ESCAPE => {
                            if (*ptr).app.escape_should_hide() {
                                hide(hwnd, ptr);
                            } else {
                                (*ptr).app.escape();
                                resize_to_results(hwnd, &*ptr);
                                let _ = InvalidateRect(Some(hwnd), None, false);
                            }
                        }
                        VK_LEFT => {
                            (*ptr).app.move_cursor_left();
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_RIGHT => {
                            (*ptr).app.move_cursor_right();
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
                        VK_RETURN => match (*ptr).app.enter() {
                            WindowAction::Quit => {
                                PostQuitMessage(0);
                            }
                            WindowAction::Hide => {
                                hide(hwnd, ptr);
                            }
                            WindowAction::Nothing => {
                                resize_to_results(hwnd, &*ptr);
                                let _ = InvalidateRect(Some(hwnd), None, false);
                            }
                        },
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_PAINT => {
                if !ptr.is_null() {
                    if let Some(r) = &(*ptr).renderer {
                        palette::draw_palette(r, &(*ptr).app, hwnd);
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
                    drop(Box::from_raw(ptr));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
