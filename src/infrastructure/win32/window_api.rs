use crate::error::AppError;
use crate::window::WindowInfo;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Win32WindowApi;

impl Win32WindowApi {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self)
    }

    pub fn get_foreground_window_info(&self) -> Result<WindowInfo, AppError> {
        let hwnd = unsafe { GetForegroundWindow() };
        self.get_window_info(hwnd)
    }

    pub fn get_window_info(&self, hwnd: HWND) -> Result<WindowInfo, AppError> {
        let hwnd_raw = hwnd.0;
        if hwnd_raw.is_null() {
            return Err(AppError::Window("invalid window handle (null)".to_string()));
        }

        let title = get_window_text(hwnd)?;
        let class = get_window_class(hwnd)?;
        let rect = get_window_rect(hwnd)?;
        let is_visible = unsafe { IsWindowVisible(hwnd).as_bool() };
        let is_minimized = unsafe { IsIconic(hwnd).as_bool() };

        Ok(WindowInfo {
            hwnd: hwnd_raw as isize,
            title,
            class,
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
            is_visible,
            is_minimized,
        })
    }

    pub fn move_window(
        &self,
        hwnd: isize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), AppError> {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        let is_minimized = unsafe { IsIconic(hwnd).as_bool() };
        if is_minimized {
            unsafe {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }

        let flags = SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW;

        unsafe {
            SetWindowPos(hwnd, HWND::default(), x, y, width, height, flags)
        }
        .map_err(|e| AppError::Win32(e))?;

        Ok(())
    }

    pub fn focus_window(&self, hwnd: isize) -> Result<(), AppError> {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        let is_minimized = unsafe { IsIconic(hwnd).as_bool() };
        if is_minimized {
            unsafe {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }

        let result = unsafe { SetForegroundWindow(hwnd) };
        if result.0 == 0 {
            let err = std::io::Error::last_os_error();
            tracing::warn!("SetForegroundWindow failed (may be normal): {}", err);
        }

        Ok(())
    }

    pub fn show_window(&self, hwnd: isize, show: bool) -> Result<(), AppError> {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);
        if show {
            unsafe { let _ = ShowWindow(hwnd, SW_RESTORE); }
            unsafe { let _ = ShowWindow(hwnd, SW_SHOW); }
        } else {
            unsafe { let _ = ShowWindow(hwnd, SW_HIDE); }
        }
        Ok(())
    }

    pub fn get_foreground_hwnd(&self) -> isize {
        unsafe { GetForegroundWindow().0 as isize }
    }

    pub fn enumerate_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        let windows: std::sync::Mutex<Vec<WindowInfo>> = std::sync::Mutex::new(Vec::new());
        let windows_ptr: *const std::sync::Mutex<Vec<WindowInfo>> = &windows;
        let lparam = LPARAM(windows_ptr as isize);

        unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if lparam.0 == 0 {
                return BOOL(1);
            }

            let windows_ptr = lparam.0 as *const std::sync::Mutex<Vec<WindowInfo>>;
            let windows = unsafe { &*windows_ptr };

            let visible = unsafe { IsWindowVisible(hwnd) };
            if visible.0 == 0 {
                return BOOL(1);
            }

            let iconic = unsafe { IsIconic(hwnd) };
            if iconic.0 != 0 {
                return BOOL(1);
            }

            let title = match get_window_text(hwnd) {
                Ok(t) => t,
                Err(_) => return BOOL(1),
            };

            if title.is_empty() {
                return BOOL(1);
            }

            let class = match get_window_class(hwnd) {
                Ok(c) => c,
                Err(_) => return BOOL(1),
            };

            let rect = match get_window_rect(hwnd) {
                Ok(r) => r,
                Err(_) => return BOOL(1),
            };

            if let Ok(mut guard) = windows.lock() {
                guard.push(WindowInfo {
                    hwnd: hwnd.0 as isize,
                    title,
                    class,
                    x: rect.left,
                    y: rect.top,
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                    is_visible: true,
                    is_minimized: false,
                });
            }

            BOOL(1)
        }

        unsafe {
            EnumWindows(Some(callback), lparam)
        }
        .map_err(|e| AppError::Win32(e))?;

        let result = windows
            .lock()
            .map_err(|e| AppError::Window(format!("mutex poisoned: {}", e)))?;

        Ok(result.clone())
    }
}

fn get_window_text(hwnd: HWND) -> Result<String, AppError> {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length == 0 {
        return Ok(String::new());
    }

    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    if copied == 0 {
        return Ok(String::new());
    }

    buffer.truncate(copied as usize);
    Ok(String::from_utf16_lossy(&buffer))
}

fn get_window_class(hwnd: HWND) -> Result<String, AppError> {
    let mut buffer: Vec<u16> = vec![0; 256];
    let copied = unsafe { GetClassNameW(hwnd, &mut buffer) };
    if copied == 0 {
        return Ok(String::new());
    }
    buffer.truncate(copied as usize);
    Ok(String::from_utf16_lossy(&buffer))
}

fn get_window_rect(hwnd: HWND) -> Result<RECT, AppError> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }
        .map_err(|e| AppError::Win32(e))?;
    Ok(rect)
}
