use crate::error::AppError;
use crate::infrastructure::win32::monitor_api::Win32MonitorApi;
use crate::layout::Rect;
use crate::monitor::MonitorInfo;

pub struct MonitorService {
    api: Win32MonitorApi,
}

impl MonitorService {
    pub fn new(api: Win32MonitorApi) -> Self {
        Self { api }
    }

    pub fn get_monitor_for_foreground_window(&self) -> Result<MonitorInfo, AppError> {
        let info = self.api.get_monitor_for_foreground_window()?;
        tracing::debug!(
            "monitor for foreground window: work_area=({},{},{},{}) primary={}",
            info.work_x, info.work_y, info.work_width, info.work_height,
            info.is_primary
        );
        Ok(info)
    }

    pub fn get_work_area_for_foreground_window(&self) -> Result<Rect, AppError> {
        let info = self.get_monitor_for_foreground_window()?;
        Ok(Rect::new(
            info.work_x,
            info.work_y,
            info.work_width,
            info.work_height,
        ))
    }
}
