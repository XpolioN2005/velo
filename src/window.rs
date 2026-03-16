use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, UpdateWindow},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
        GetSystemMetrics, GetWindowLongPtrW, IDC_ARROW, LoadCursorW, MSG, PostQuitMessage,
        RegisterClassExW, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SWP_NOMOVE, SWP_NOZORDER,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, WM_CHAR, WM_CLOSE,
        WM_DESTROY, WM_KEYDOWN, WM_KILLFOCUS, WM_PAINT, WM_SETFOCUS, WM_SIZE, WNDCLASSEXW,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
    },
    core::*,
};

use crate::renderer::{Renderer, layout, palette};
use crate::{app::AppState, command::ExecuteResult};

const WIN_WIDTH_RATIO: f32 = 0.40;
const WIN_TOP_RATIO: f32 = 0.08;

const VK_BACK: u32 = 0x08;
const VK_ESCAPE: u32 = 0x1B;
const VK_RETURN: u32 = 0x0D;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;

struct WindowState {
    renderer: Renderer,
    app: AppState,
    win_w: i32,
}

unsafe fn resize_to_results(hwnd: HWND, state: &WindowState) {
    unsafe {
        let h = layout::window_height(state.app.results.len());
        let _ = SetWindowPos(hwnd, None, 0, 0, state.win_w, h, SWP_NOMOVE | SWP_NOZORDER);
    }
}

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
        let win_x = (screen_w - win_w) / 2;
        let win_y = (screen_h as f32 * WIN_TOP_RATIO) as i32;

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
            renderer: Renderer::new(hwnd).unwrap(),
            app: AppState::new(),
            win_w,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;

        match msg {
            WM_SIZE => {
                if !ptr.is_null() {
                    let w = (lparam.0 & 0xFFFF) as u32;
                    let h = ((lparam.0 >> 16) & 0xFFFF) as u32;
                    if w > 0 && h > 0 {
                        let _ = (*ptr).renderer.resize(w, h);
                    }
                }
                LRESULT(0)
            }
            WM_SETFOCUS => {
                if !ptr.is_null() {
                    (*ptr).app.focused = true;
                    InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                if !ptr.is_null() {
                    (*ptr).app.focused = false;
                    InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }
            WM_CHAR => {
                if !ptr.is_null() {
                    if let Some(c) = char::from_u32(wparam.0 as u32) {
                        if !c.is_control() {
                            (*ptr).app.push_char(c);
                            resize_to_results(hwnd, &*ptr);
                            InvalidateRect(Some(hwnd), None, false);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                if !ptr.is_null() {
                    match wparam.0 as u32 {
                        VK_BACK => {
                            (*ptr).app.pop_char();
                            resize_to_results(hwnd, &*ptr);
                            InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_ESCAPE => {
                            (*ptr).app.clear_query();
                            resize_to_results(hwnd, &*ptr);
                            let _ = ShowWindow(hwnd, SW_HIDE);
                        }
                        VK_UP => {
                            (*ptr).app.select_prev();
                            InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_DOWN => {
                            (*ptr).app.select_next();
                            InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_RETURN => {
                            match (*ptr).app.execute_selected() {
                                ExecuteResult::Quit => {
                                    PostQuitMessage(0);
                                }
                                ExecuteResult::Hide => {
                                    (*ptr).app.clear_query();
                                    resize_to_results(hwnd, &*ptr);
                                    let _ = ShowWindow(hwnd, SW_HIDE);
                                }
                                ExecuteResult::ReloadConfig => { /* Step 8 */ }
                                ExecuteResult::Nothing => {}
                            }
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_PAINT => {
                if !ptr.is_null() {
                    let state = &*ptr;
                    palette::draw_palette(&state.renderer, &state.app, hwnd);
                }
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_CLOSE => {
                if !ptr.is_null() {
                    (*ptr).app.clear_query();
                    resize_to_results(hwnd, &*ptr);
                }
                let _ = ShowWindow(hwnd, SW_HIDE);
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
