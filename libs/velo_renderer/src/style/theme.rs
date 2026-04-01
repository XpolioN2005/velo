use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

pub struct Theme {
    pub bg: D2D1_COLOR_F,
    pub text: D2D1_COLOR_F,
    pub highlight: D2D1_COLOR_F,
    pub input_bg: D2D1_COLOR_F,
    pub cursor: D2D1_COLOR_F,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: D2D1_COLOR_F {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            text: D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            highlight: D2D1_COLOR_F {
                r: 0.2,
                g: 0.4,
                b: 0.8,
                a: 1.0,
            },
            input_bg: D2D1_COLOR_F {
                r: 0.15,
                g: 0.15,
                b: 0.15,
                a: 1.0,
            },
            cursor: D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        }
    }
}
