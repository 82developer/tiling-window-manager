use crate::error::AppError;
use crate::layout::{LayoutEngine, LayoutType};
use crate::services::monitor_service::MonitorService;
use crate::services::window_service::WindowService;

pub struct LayoutService {
    engine: LayoutEngine,
    window_service: WindowService,
    monitor_service: MonitorService,
}

impl LayoutService {
    pub fn new(
        gap: i32,
        margin: i32,
        window_service: WindowService,
        monitor_service: MonitorService,
    ) -> Self {
        Self {
            engine: LayoutEngine::new(gap, margin),
            window_service,
            monitor_service,
        }
    }

    pub fn apply_layout(&self, layout_type: LayoutType) -> Result<(), AppError> {
        let window = self.window_service.get_foreground_window()?;
        let work_area = self.monitor_service.get_work_area_for_foreground_window()?;

        let target_rect = self.engine.calculate(layout_type, work_area);

        tracing::info!(
            "applying layout {:?} to window '{}' (class={}): rect=({},{},{},{})",
            layout_type,
            window.title,
            window.class,
            target_rect.x,
            target_rect.y,
            target_rect.width,
            target_rect.height,
        );

        self.window_service.move_window(
            window.hwnd,
            target_rect.x,
            target_rect.y,
            target_rect.width,
            target_rect.height,
        )
    }

    pub fn get_foreground_hwnd(&self) -> isize {
        self.window_service.get_foreground_hwnd()
    }

    pub fn move_window_rel(
        &self,
        _hwnd: isize,
        dx: i32,
        dy: i32,
    ) -> Result<(), AppError> {
        let window = self.window_service.get_foreground_window()?;
        let work_area = self.monitor_service.get_work_area_for_foreground_window()?;
        let new_x = clamp(window.x + dx, work_area.x, work_area.x + work_area.width - 100);
        let new_y = clamp(window.y + dy, work_area.y, work_area.y + work_area.height - 100);
        self.window_service.move_window(window.hwnd, new_x, new_y, window.width, window.height)
    }

    pub fn apply_move(
        &self,
        dx: i32,
        dy: i32,
        dw: i32,
        dh: i32,
    ) -> Result<(), AppError> {
        let window = self.window_service.get_foreground_window()?;
        let work_area = self.monitor_service.get_work_area_for_foreground_window()?;

        let new_x = clamp(window.x + dx, work_area.x, work_area.x + work_area.width - 100);
        let new_y = clamp(window.y + dy, work_area.y, work_area.y + work_area.height - 100);
        let new_w = (window.width + dw).max(100).min(work_area.width);
        let new_h = (window.height + dh).max(100).min(work_area.height);

        tracing::info!(
            "moving window '{}': ({},{}) {}x{} -> ({},{}) {}x{}",
            window.title,
            window.x, window.y, window.width, window.height,
            new_x, new_y, new_w, new_h,
        );

        self.window_service.move_window(window.hwnd, new_x, new_y, new_w, new_h)
    }
}

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}
