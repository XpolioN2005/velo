use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, RegisterHotKey, VK_P};
use windows::Win32::{
    Foundation::HINSTANCE, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
};
use windows::core::*;

use crate::app::AppState;
use crate::renderer::Renderer;
use crate::renderer::layout;
use crate::window::{state::*, wnd_proc::wnd_proc};

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
            caret_visible: true,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        let _ = UpdateWindow(hwnd);
        let _ = RegisterHotKey(Some(hwnd), HOTKEY_ID, MOD_CONTROL | MOD_ALT, VK_P.0 as u32);
        let _ = SetTimer(Some(hwnd), CARET_TIMER_ID, CARET_BLINK_MS, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
