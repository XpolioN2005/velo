pub struct Dimensions {
    pub padding: f32,
    pub item_height: f32,
    pub spacing: f32,
    pub width: f32,
    pub input_height: f32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            padding: 10.0,
            item_height: 32.0,
            spacing: 4.0,
            width: 500.0,
            input_height: 40.0,
        }
    }
}
