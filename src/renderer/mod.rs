use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct2D::Common::{
        D2D_SIZE_U, D2D1_ALPHA_MODE_UNKNOWN, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
    },
    Win32::Graphics::Direct2D::{
        D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
        D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES, D2D1CreateFactory, ID2D1Factory,
        ID2D1HwndRenderTarget,
    },
    Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
    Win32::UI::WindowsAndMessaging::GetClientRect,
    core::*,
};

pub struct Renderer {
    pub factory: ID2D1Factory,
    pub target: ID2D1HwndRenderTarget,
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            // Create the D2D factory — single threaded since we have one thread
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

            // Get window size for the render target
            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc)?;

            let size = D2D_SIZE_U {
                width: (rc.right - rc.left) as u32,
                height: (rc.bottom - rc.top) as u32,
            };

            // Render target properties — default pixel format, DPI
            let rtp = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: Default::default(),
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_UNKNOWN,
                    alphaMode: D2D1_ALPHA_MODE_UNKNOWN,
                },
                dpiX: 0.0, // 0 = use system DPI
                dpiY: 0.0,
                usage: Default::default(),
                minLevel: Default::default(),
            };

            // HWND render target — draws directly into our window
            let hwnd_rtp = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: size,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let target = factory.CreateHwndRenderTarget(&rtp, &hwnd_rtp)?;

            Ok(Self { factory, target })
        }
    }

    /// Call at the start of every WM_PAINT
    pub fn begin(&self) {
        unsafe { self.target.BeginDraw() }
    }

    /// Call at the end of every WM_PAINT
    /// Returns Err if the render target was lost (e.g. display change)
    pub fn end(&self) -> Result<()> {
        unsafe { self.target.EndDraw(None, None) }
    }

    /// Clear the entire target to a solid color
    pub fn clear(&self) {
        unsafe {
            self.target.Clear(Some(&D2D1_COLOR_F {
                r: 0.10,
                g: 0.10,
                b: 0.11,
                a: 1.0,
            }))
        }
    }
}
