use crate::error::AppError;
use crate::monitor::MonitorInfo;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

const MONITORINFOF_PRIMARY: u32 = 0x00000001;

pub struct Win32MonitorApi;

impl Win32MonitorApi {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self)
    }

    pub fn get_monitor_for_foreground_window(&self) -> Result<MonitorInfo, AppError> {
        let hwnd = unsafe { GetForegroundWindow() };
        self.get_monitor_for_window(hwnd.0 as isize)
    }

    pub fn get_monitor_for_window(&self, hwnd: isize) -> Result<MonitorInfo, AppError> {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);
        let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };

        let mut monitor_info = MONITORINFO::default();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let result = unsafe { GetMonitorInfoW(monitor, &mut monitor_info) };
        if result.0 == 0 {
            let err = std::io::Error::last_os_error();
            return Err(AppError::Monitor(format!("GetMonitorInfoW failed: {}", err)));
        }

        let rc_monitor = &monitor_info.rcMonitor;
        let rc_work = &monitor_info.rcWork;
        let is_primary = (monitor_info.dwFlags & MONITORINFOF_PRIMARY) != 0;

        Ok(MonitorInfo {
            handle: monitor.0 as isize,
            x: rc_monitor.left,
            y: rc_monitor.top,
            width: rc_monitor.right - rc_monitor.left,
            height: rc_monitor.bottom - rc_monitor.top,
            work_x: rc_work.left,
            work_y: rc_work.top,
            work_width: rc_work.right - rc_work.left,
            work_height: rc_work.bottom - rc_work.top,
            is_primary,
        })
    }
}
