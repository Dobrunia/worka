// Prevent multiple instances using Windows mutex
use std::sync::Mutex;

// Use SendWrapper to make HANDLE Send between threads
#[allow(dead_code)]
struct SendHandle {
    handle: Option<windows::Win32::Foundation::HANDLE>,
}

unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

static MUTEX_HANDLE: Mutex<Option<SendHandle>> = Mutex::new(None);

#[cfg(target_os = "windows")]
fn init_single_instance() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Foundation::GetLastError;
    use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
    use windows::Win32::System::Threading::CreateMutexW;

    let mutex_name: Vec<u16> = OsStr::new("Local\\WorkaSingleInstanceMutex")
        .encode_wide()
        .chain(Some(0))
        .collect();

    unsafe {
        // bInitialOwner = false: mutex is a sentinel only, we don't need to own it.
        let handle = CreateMutexW(None, false, windows::core::PCWSTR(mutex_name.as_ptr()));
        if handle.is_ok() {
            let last_error = GetLastError();
            if last_error == ERROR_ALREADY_EXISTS {
                let _ = CloseHandle(handle.unwrap());
                return false; // Already running, this instance should exit
            }
            
            // Store handle globally to keep mutex alive
            *MUTEX_HANDLE.lock().unwrap() = Some(SendHandle { handle: Some(handle.unwrap()) });
            return true; // First instance
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn init_single_instance() -> bool {
    true
}

fn main() {
    // Check if another instance is already running
    if !init_single_instance() {
        // Exit silently - the running instance continues
        std::process::exit(0);
    }

    tauri_app_lib::run();
}
