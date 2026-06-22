use crate::config::IgnoreConfig;
use crate::error::AppError;
use crate::infrastructure::win32::window_api::Win32WindowApi;
use crate::window::WindowInfo;

pub struct WindowService {
    api: Win32WindowApi,
    ignore_config: IgnoreConfig,
}

impl WindowService {
    pub fn new(api: Win32WindowApi, ignore_config: IgnoreConfig) -> Self {
        Self { api, ignore_config }
    }

    pub fn get_foreground_window(&self) -> Result<WindowInfo, AppError> {
        let info = self.api.get_foreground_window_info()?;
        if self.is_ignored(&info) {
            return Err(AppError::Window(format!(
                "window is ignored: class='{}' title='{}'",
                info.class, info.title
            )));
        }
        Ok(info)
    }

    pub fn get_foreground_hwnd(&self) -> isize {
        self.api.get_foreground_hwnd()
    }

    pub fn move_window(
        &self, hwnd: isize, x: i32, y: i32, width: i32, height: i32,
    ) -> Result<(), AppError> {
        tracing::debug!("moving window hwnd={} to x={} y={} w={} h={}", hwnd, x, y, width, height);
        self.api.move_window(hwnd, x, y, width, height)
    }

    pub fn focus_window(&self, hwnd: isize) -> Result<(), AppError> {
        tracing::debug!("focusing window hwnd={}", hwnd);
        self.api.focus_window(hwnd)
    }

    pub fn show_window(&self, hwnd: isize, show: bool) -> Result<(), AppError> {
        self.api.show_window(hwnd, show)
    }

    pub fn enumerate_visible_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        let windows = self.api.enumerate_windows()?;
        let filtered: Vec<WindowInfo> = windows
            .into_iter()
            .filter(|w| !self.is_ignored(w))
            .collect();
        Ok(filtered)
    }

    pub fn is_ignored(&self, info: &WindowInfo) -> bool {
        if self.ignore_config.classes.contains(&info.class) {
            return true;
        }
        if self.ignore_config.titles.contains(&info.title) {
            return true;
        }
        if info.title.is_empty() && info.class.is_empty() {
            return true;
        }
        false
    }
}
