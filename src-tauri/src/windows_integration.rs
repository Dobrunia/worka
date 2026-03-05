use crate::state::INPUT_COUNTERS;
use std::thread;

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use base64::Engine;
    use image::ImageFormat;
    use std::ffi::OsStr;
    use std::io::Cursor;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::{
        Foundation::{CloseHandle, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS,
            DeleteDC, DeleteObject, GetDIBits, GetObjectW, HGDIOBJ,
        },
        Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
        System::{
            SystemInformation::GetTickCount64,
            Threading::{
                OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
        },
        UI::{
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
            Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON, SHGetFileInfoW},
            WindowsAndMessaging::{
                CallNextHookEx, DestroyIcon, GetForegroundWindow, GetIconInfo, GetMessageW,
                GetWindowTextW, GetWindowThreadProcessId, ICONINFO, MSG, SetWindowsHookExW,
                UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN,
                WM_MBUTTONDOWN, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_SYSKEYDOWN,
            },
        },
    };

    unsafe extern "system" fn keyboard_hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code >= 0 {
            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                if let Some(c) = INPUT_COUNTERS.get() {
                    c.keyboard
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    unsafe extern "system" fn mouse_hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code >= 0 {
            match wparam.0 as u32 {
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                    if let Some(c) = INPUT_COUNTERS.get() {
                        c.mouse_clicks
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                WM_MOUSEWHEEL => {
                    if let Some(c) = INPUT_COUNTERS.get() {
                        c.mouse_wheel
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                _ => {}
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    pub fn spawn_input_hook_thread() {
        thread::spawn(|| unsafe {
            let kb_hook =
                match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("Failed to install keyboard hook: {e:?}");
                        return;
                    }
                };

            let ms_hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Failed to install mouse hook: {e:?}");
                    let _ = UnhookWindowsHookEx(kb_hook);
                    return;
                }
            };

            // Message pump — required for LL hooks to receive events.
            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, None, 0, 0);
                if ret.0 <= 0 {
                    break; // 0 = WM_QUIT, -1 = error
                }
            }

            let _ = UnhookWindowsHookEx(kb_hook);
            let _ = UnhookWindowsHookEx(ms_hook);
        });
    }

    pub fn get_idle_time() -> u64 {
        unsafe {
            let mut last_input = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            if GetLastInputInfo(&mut last_input).as_bool() {
                let now = GetTickCount64();
                (now.saturating_sub(last_input.dwTime as u64)) / 1000
            } else {
                0
            }
        }
    }

    pub fn get_active_window_info() -> Option<(String, String)> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            let process_handle =
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
            let mut name_buf = [0u16; 1024];
            let mut name_len = name_buf.len() as u32;
            let query_result = QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
            );
            let _ = CloseHandle(process_handle);
            query_result.ok()?;
            let exe_path = String::from_utf16_lossy(&name_buf[..name_len as usize]).to_string();

            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            let title = String::from_utf16_lossy(&title_buf[..len as usize]).to_string();

            Some((exe_path, title))
        }
    }

    pub fn extract_app_icon_data_url(exe_path: &str) -> Option<String> {
        unsafe {
            let path_wide: Vec<u16> = OsStr::new(exe_path)
                .encode_wide()
                .chain(Some(0))
                .collect();

            let mut file_info = SHFILEINFOW::default();
            let result = SHGetFileInfoW(
                windows::core::PCWSTR(path_wide.as_ptr()),
                FILE_ATTRIBUTE_NORMAL,
                Some(&mut file_info),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON,
            );

            if result == 0 || file_info.hIcon.0.is_null() {
                return None;
            }

            let png_data_url = icon_handle_to_png_data_url(file_info.hIcon);
            let _ = DestroyIcon(file_info.hIcon);
            png_data_url
        }
    }

    unsafe fn icon_handle_to_png_data_url(icon_handle: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<String> {
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(icon_handle, &mut icon_info).is_err() {
            return None;
        }

        let bitmap_handle = if icon_info.hbmColor.0.is_null() {
            icon_info.hbmMask
        } else {
            icon_info.hbmColor
        };

        let mut bitmap = BITMAP::default();
        if GetObjectW(
            HGDIOBJ(bitmap_handle.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some((&mut bitmap as *mut BITMAP).cast()),
        ) == 0
        {
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
            return None;
        }

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight.abs();
        if width <= 0 || height <= 0 {
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
            return None;
        }

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bgra_data = vec![0u8; (width * height * 4) as usize];
        let hdc = CreateCompatibleDC(None);
        if hdc.0.is_null() {
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
            return None;
        }

        let copied = GetDIBits(
            hdc,
            bitmap_handle,
            0,
            height as u32,
            Some(bgra_data.as_mut_ptr().cast()),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(hdc);
        let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));
        let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));

        if copied == 0 {
            return None;
        }

        for rgba in bgra_data.chunks_exact_mut(4) {
            rgba.swap(0, 2); // BGRA -> RGBA
        }

        let image = image::RgbaImage::from_raw(width as u32, height as u32, bgra_data)?;
        let mut png_bytes = Cursor::new(Vec::new());
        image.write_to(&mut png_bytes, ImageFormat::Png).ok()?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(png_bytes.into_inner());
        Some(format!("data:image/png;base64,{encoded}"))
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn spawn_input_hook_thread() {}

    pub fn get_idle_time() -> u64 {
        0
    }

    pub fn get_active_window_info() -> Option<(String, String)> {
        None
    }

    pub fn extract_app_icon_data_url(_exe_path: &str) -> Option<String> {
        None
    }
}

pub use imp::{
    extract_app_icon_data_url, get_active_window_info, get_idle_time, spawn_input_hook_thread,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_idle_time_does_not_panic() {
        let _ = get_idle_time();
    }

    #[test]
    fn should_get_active_window_info_does_not_panic() {
        let result = std::panic::catch_unwind(get_active_window_info);
        assert!(result.is_ok());
    }
}
