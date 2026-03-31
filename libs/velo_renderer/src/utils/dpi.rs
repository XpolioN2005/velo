pub fn scale(value: f32, dpi: f32) -> f32 {
    value * (dpi / 96.0)
}
