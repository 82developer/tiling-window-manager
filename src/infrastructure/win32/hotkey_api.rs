use crate::error::AppError;
use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const TRAY_ICON_ID: u32 = 1;
const WM_TRAY_CALLBACK: u32 = WM_USER + 100;
const IDM_EXIT: u32 = 1001;

extern "system" {
    fn RegisterHotKey(hwnd: *mut std::ffi::c_void, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hwnd: *mut std::ffi::c_void, id: i32) -> i32;
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TRAY_CALLBACK {
        let event = lparam.0 as u32;
        if event == WM_RBUTTONUP || event == WM_RBUTTONDOWN {
            show_tray_menu(hwnd);
        }
        return LRESULT(0);
    }
    if msg == WM_COMMAND {
        let cmd = (wparam.0 & 0xFFFF) as u32;
        if cmd == IDM_EXIT {
            PostQuitMessage(0);
            return LRESULT(0);
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let hmenu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(_) => return,
    };
    if hmenu.0.is_null() {
        return;
    }

    let _ = AppendMenuW(hmenu, MF_STRING, IDM_EXIT as usize, w!("E&xit"));

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);

    let _ = SetForegroundWindow(hwnd);

    let _ = TrackPopupMenu(
        hmenu,
        TPM_LEFTALIGN | TPM_BOTTOMALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        None,
    );

    let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(hmenu);
}

pub struct Win32HotkeyApi {
    registered_ids: Vec<i32>,
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl Win32HotkeyApi {
    pub fn new() -> Result<Self, AppError> {
        unsafe {
            let hmodule = GetModuleHandleW(None)
                .map_err(|e| AppError::Win32(e))?;
            let hinstance = HINSTANCE(hmodule.0);

            let class_name = w!("RTWM_TrayWindow");

            let wc = WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance,
                lpszClassName: class_name,
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            if atom == 0 {
                return Err(AppError::General("RegisterClassW failed".to_string()));
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                class_name,
                WS_POPUP,
                0,
                0,
                0,
                0,
                HWND::default(),
                HMENU::default(),
                hinstance,
                None,
            )?;

            let hicon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();

            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ICON_ID;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAY_CALLBACK;
            nid.hIcon = hicon;

            let tip = "Rust Tiling Window Manager";
            let tip_wide: Vec<u16> = tip.encode_utf16().take(127).collect();
            let len = tip_wide.len();
            nid.szTip[..len].copy_from_slice(&tip_wide);
            nid.szTip[len] = 0;

            if Shell_NotifyIconW(NIM_ADD, &nid) == false {
                return Err(AppError::General(
                    "Shell_NotifyIconW NIM_ADD failed".to_string(),
                ));
            }

            tracing::info!("System tray icon added (hwnd={:?})", hwnd.0);

            Ok(Self {
                registered_ids: Vec::new(),
                hwnd,
                nid,
            })
        }
    }

    pub fn register_hotkey(
        &mut self,
        id: i32,
        modifiers: u32,
        vk: u32,
    ) -> Result<(), AppError> {
        let result = unsafe { RegisterHotKey(std::ptr::null_mut(), id, modifiers, vk) };

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
                let should_continue = handler(hotkey_id);
                if !should_continue {
                    break;
                }
            } else {
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
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
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
