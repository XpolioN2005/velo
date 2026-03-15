use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, UpdateWindow},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
        GetSystemMetrics, GetWindowLongPtrW, IDC_ARROW, LoadCursorW, MSG, PostQuitMessage,
        RegisterClassExW, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SetWindowLongPtrW, ShowWindow,
        TranslateMessage, WM_CLOSE, WM_DESTROY, WM_PAINT, WNDCLASSEXW, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
    },
    core::*,
};

use crate::renderer::Renderer;

const WIN_WIDTH_RATIO: f32 = 0.40;
const WIN_MAX_H_RATIO: f32 = 0.33;
const WIN_TOP_RATIO: f32 = 0.08;
const QUERY_BAR_HEIGHT: i32 = 48;

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
            // No background brush — Direct2D owns all painting now
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

        // Create renderer and store it on the window
        let renderer = Box::new(Renderer::new(hwnd).unwrap());
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(renderer) as isize);

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
        match msg {
            WM_PAINT => {
                // Retrieve renderer from window userdata
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;
                if !ptr.is_null() {
                    let renderer = &*ptr;
                    renderer.begin();
                    renderer.clear();
                    renderer.end().unwrap();
                }

                // Tell Win32 the paint request is satisfied
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);

                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_DESTROY => {
                // Clean up renderer before exit
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;
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
