// Single-instance guard via Windows named mutex.
use std::sync::Mutex;

#[allow(dead_code)]
struct SendHandle {
    handle: Option<windows::Win32::Foundation::HANDLE>,
}
unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

static MUTEX_HANDLE: Mutex<Option<SendHandle>> = Mutex::new(None);

/// Try to create the named mutex and become the owner.
/// Returns true if this is the first (or only) instance.
#[cfg(target_os = "windows")]
fn try_acquire_mutex() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;

    let name: Vec<u16> = OsStr::new("Local\\WorkaSingleInstanceMutex")
        .encode_wide()
        .chain(Some(0))
        .collect();

    unsafe {
        let handle = CreateMutexW(None, false, windows::core::PCWSTR(name.as_ptr()));
        if let Ok(h) = handle {
            if GetLastError() != ERROR_ALREADY_EXISTS {
                // We are the first instance — keep the handle alive for the process lifetime.
                *MUTEX_HANDLE.lock().unwrap() = Some(SendHandle { handle: Some(h) });
                return true;
            }
            let _ = CloseHandle(h);
        }
        false
    }
}

#[cfg(target_os = "windows")]
fn init_single_instance() -> bool {
    if try_acquire_mutex() {
        return true;
    }

    // Another instance already holds the mutex.

    // Release builds: just exit — the running instance is the correct one.
    #[cfg(not(debug_assertions))]
    return false;

    // Debug builds (dev mode): the leftover process is stale (e.g. survived a
    // Ctrl-C from `tauri dev`).  Kill it and retry so every `npm run dev` always
    // starts with a clean slate — no white screen from a dead Vite connection.
    #[cfg(debug_assertions)]
    return {
        let current_pid = std::process::id().to_string();
        let _ = std::process::Command::new("taskkill")
            .args([
                "/F",
                "/IM",
                "worka.exe",
                "/FI",
                &format!("PID ne {}", current_pid),
            ])
            .output();
        std::thread::sleep(std::time::Duration::from_millis(300));
        try_acquire_mutex()
    };
}

#[cfg(not(target_os = "windows"))]
fn init_single_instance() -> bool {
    true
}

fn main() {
    if !init_single_instance() {
        std::process::exit(0);
    }
    tauri_app_lib::run();
}
