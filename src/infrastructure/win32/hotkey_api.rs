use crate::error::AppError;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

extern "system" {
    fn RegisterHotKey(hwnd: *mut std::ffi::c_void, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hwnd: *mut std::ffi::c_void, id: i32) -> i32;
}

pub struct Win32HotkeyApi {
    registered_ids: Vec<i32>,
}

impl Win32HotkeyApi {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            registered_ids: Vec::new(),
        })
    }

    pub fn register_hotkey(
        &mut self,
        id: i32,
        modifiers: u32,
        vk: u32,
    ) -> Result<(), AppError> {
        let result = unsafe {
            RegisterHotKey(std::ptr::null_mut(), id, modifiers, vk)
        };

        if result == 0 {
            let err = std::io::Error::last_os_error();
            return Err(AppError::Hotkey(format!(
                "failed to register hotkey id={} mods={:#x} vk={:#x}: {}",
                id, modifiers, vk, err
            )));
        }

        self.registered_ids.push(id);

        Ok(())
    }

    pub fn unregister_all(&self) {
        for id in &self.registered_ids {
            unsafe {
                let _ = UnregisterHotKey(std::ptr::null_mut(), *id);
            }
        }
    }

    pub fn run_message_loop<F>(&self, mut handler: F) -> Result<(), AppError>
    where
        F: FnMut(i32) -> bool,
    {
        let mut msg = MSG::default();

        loop {
            let ret = unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0) };

            if ret.0 == 0 {
                tracing::info!("WM_QUIT received, exiting message loop");
                break;
            }

            if ret.0 == -1 {
                let err = std::io::Error::last_os_error();
                tracing::error!("GetMessageW error: {}", err);
                return Err(AppError::Win32(windows::core::Error::from_win32()));
            }

            if msg.message == WM_HOTKEY {
                let hotkey_id = msg.wParam.0 as i32;
                tracing::debug!("WM_HOTKEY received: id={}", hotkey_id);

                let should_continue = handler(hotkey_id);
                if !should_continue {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn request_quit(&self) {
        unsafe {
            PostQuitMessage(0);
        }
    }
}

impl Drop for Win32HotkeyApi {
    fn drop(&mut self) {
        self.unregister_all();
    }
}
