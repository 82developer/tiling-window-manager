#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub handle: isize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub work_x: i32,
    pub work_y: i32,
    pub work_width: i32,
    pub work_height: i32,
    pub is_primary: bool,
}

impl MonitorInfo {
    pub fn work_area(&self) -> (i32, i32, i32, i32) {
        (self.work_x, self.work_y, self.work_width, self.work_height)
    }
}
