// types.rs

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Color(pub u8, pub u8, pub u8);