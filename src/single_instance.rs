#[cfg(target_os = "windows")]
mod windows_impl {
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE, CloseHandle};
    use windows::Win32::System::Threading::CreateMutexW; // Import the Threading module
    use windows::core::{HSTRING, PCWSTR};

    // A unique name for your mutex.
    // The "Global\\" prefix ensures it's a system-wide mutex.
    const MUTEX_NAME_STR: &str = "Global\\SyncthingersInstanceMutex";

    // We need to store the handle so it stays alive for the duration of the app.
    // If it's dropped, the mutex is released.
    // Using a static variable to hold the mutex handle.
    // This is a bit simplified; in a larger app, you might manage this handle
    // within an application state struct that's kept alive.
    static mut MUTEX_HANDLE: Option<HANDLE> = None;

    pub fn ensure_single_instance() -> Result<(), String> {
        let mutex_name_hstring = HSTRING::from(MUTEX_NAME_STR);
        let mutex_name_pcwstr = PCWSTR(mutex_name_hstring.as_ptr());

        let handle: HANDLE;
        unsafe {
            // CreateMutexW returns a Result<HANDLE, Error>.
            // We use `map_err` to convert the windows_core::Error into a String,
            // and then `?` to propagate the Err(String) or unwrap the Ok(HANDLE).
            handle = CreateMutexW(None, true, mutex_name_pcwstr).map_err(|e| e.to_string())?;

            if handle.is_invalid() {
                let err = GetLastError();
                return Err(format!(
                    "Failed to create or open mutex. Windows Error: {:?}",
                    err
                ));
            }

            if GetLastError() == ERROR_ALREADY_EXISTS {
                // Another instance is already running.
                // We still got a handle to the existing mutex, so close it.
                let _ = CloseHandle(handle); // Use imported CloseHandle
                return Err("Syncthingers is already running.".to_string());
            }
            // If we reach here, we are the first instance and we own the mutex.
            // Store the handle to keep the mutex alive.
            MUTEX_HANDLE = Some(handle);
        }
        Ok(())
    }

    // Optional: A function to explicitly release the mutex on shutdown,
    // though the OS will do it when the process exits if the handle is still open.
    // pub fn release_mutex_on_shutdown() {
    //     unsafe {
    //         if let Some(handle) = MUTEX_HANDLE.take() {
    //             CloseHandle(handle);
    //         }
    //     }
    // }
}

#[cfg(target_os = "windows")]
pub use windows_impl::ensure_single_instance;

// Placeholder for other platforms
#[cfg(not(target_os = "windows"))]
pub fn ensure_single_instance() -> Result<(), String> {
    // For non-Windows platforms, this might use a lock file or another mechanism.
    // For now, we'll just allow multiple instances or return Ok.
    println!("Warning: Single instance check not implemented for this platform.");
    Ok(())
}