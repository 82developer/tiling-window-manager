#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub index: usize,
    pub name: String,
    pub windows: Vec<isize>,
}

impl Workspace {
    pub fn new(index: usize, name: &str) -> Self {
        Self {
            index,
            name: name.to_string(),
            windows: Vec::new(),
        }
    }

    pub fn add_window(&mut self, hwnd: isize) {
        if !self.windows.contains(&hwnd) {
            self.windows.push(hwnd);
        }
    }

    pub fn remove_window(&mut self, hwnd: isize) {
        self.windows.retain(|h| *h != hwnd);
    }
}
