use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, UpdateWindow},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
        GetSystemMetrics, GetWindowLongPtrW, IDC_ARROW, LoadCursorW, MSG, PostQuitMessage,
        RegisterClassExW, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SetWindowLongPtrW, ShowWindow,
        TranslateMessage, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KILLFOCUS, WM_PAINT,
        WM_SETFOCUS, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
    },
    core::*,
};

use crate::app::AppState;
use crate::renderer::{Renderer, palette};

const WIN_WIDTH_RATIO: f32 = 0.40;
const WIN_MAX_H_RATIO: f32 = 0.33;
const WIN_TOP_RATIO: f32 = 0.08;
const QUERY_BAR_HEIGHT: i32 = 48;

const VK_BACK: u32 = 0x08;
const VK_ESCAPE: u32 = 0x1B;
const VK_RETURN: u32 = 0x0D;

struct WindowState {
    renderer: Renderer,
    app: AppState,
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
        let win_h = QUERY_BAR_HEIGHT.min((screen_h as f32 * WIN_MAX_H_RATIO) as i32);
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
                            InvalidateRect(Some(hwnd), None, false);
                        }
                        VK_ESCAPE => {
                            (*ptr).app.clear_query();
                            let _ = ShowWindow(hwnd, SW_HIDE);
                        }
                        VK_RETURN => {
                            // Step 5: will trigger command execution
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
