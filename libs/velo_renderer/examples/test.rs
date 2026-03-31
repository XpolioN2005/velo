use velo_renderer::{Renderer, ResultItem};

use windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let renderer = Box::new(Renderer::new(hwnd).unwrap());
                let ptr = Box::into_raw(renderer);

                SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);

                LRESULT(0)
            }

            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);

                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;

                if !ptr.is_null() {
                    let renderer = &*ptr;

                    let items = vec![
                        ResultItem {
                            title: "First".into(),
                            selected: true,
                        },
                        ResultItem {
                            title: "Second".into(),
                            selected: false,
                        },
                        ResultItem {
                            title: "Third".into(),
                            selected: false,
                        },
                    ];

                    renderer.begin();
                    renderer.draw_results(&items);
                    renderer.end();
                }

                EndPaint(hwnd, &ps);

                // Force redraw (so you actually see updates later)
                InvalidateRect(Some(hwnd), None, false);

                LRESULT(0)
            }

            WM_SIZE => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;

                if !ptr.is_null() {
                    let renderer = &*ptr;

                    let width = (lparam.0 & 0xFFFF) as u32;
                    let height = ((lparam.0 >> 16) & 0xFFFF) as u32;

                    renderer.resize(width, height);
                }

                LRESULT(0)
            }

            WM_DESTROY => {
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

fn main() -> Result<()> {
    unsafe {
        let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);

        let class_name = w!("test_window");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: hinstance,
            lpszClassName: class_name,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let _ = CreateWindowExW(
            Default::default(),
            class_name,
            w!("Renderer Test"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            None,
            None,
            Some(hinstance),
            None,
        );

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}
