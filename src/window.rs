#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_visible: bool,
    pub is_minimized: bool,
}

impl WindowInfo {
    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}
