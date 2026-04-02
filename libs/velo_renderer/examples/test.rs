use velo_renderer::{Renderer, ResultItem};

use windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

const ENABLE_RENDERER: bool = true;
const ENABLE_DRAW: bool = true;
const ENABLE_LOOP: bool = true; // InvalidateRect

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                log_memory("WM_CREATE start");

                if ENABLE_RENDERER {
                    let renderer = Box::new(Renderer::new(hwnd).unwrap());
                    let ptr = Box::into_raw(renderer);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);

                    log_memory("after renderer init");
                }

                LRESULT(0)
            }

            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);

                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;

                if ENABLE_RENDERER && !ptr.is_null() {
                    let renderer = &*ptr;

                    if ENABLE_DRAW {
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
                        renderer.end();
                    }
                }

                let _ = EndPaint(hwnd, &ps);

                if ENABLE_LOOP {
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }

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
    log_memory("start");

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

        log_memory("before window");

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

        log_memory("after window");

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}
use sysinfo::{Pid, System};

fn log_memory(stage: &str) {
    let mut sys = System::new_all();
    sys.refresh_all();

    let pid = Pid::from_u32(std::process::id());

    if let Some(proc) = sys.process(pid) {
        let mem_mb = proc.memory() as f64 / 1024.0 / 1024.0;

        println!("[{}] Memory: {:.2} MB", stage, mem_mb);
    }
}
