use log::{debug, error, info};
use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicBool, Ordering};
use winapi::um::winuser::{FindWindowA, SendMessageA, WM_USER};
use winapi::shared::windef::HWND;
use std::ffi::CString;

// Unique message ID for inter-process communication
const WM_APP_SHOW_WINDOW: u32 = WM_USER + 1;

// Static flag to track singleton status
static SINGLETON_CREATED: AtomicBool = AtomicBool::new(false);

/// Ensures that only one instance of the application is running.
/// 
/// Returns:
///   - Ok(true) if this is the first instance
///   - Ok(false) if another instance was found (and a message was sent to it)
///   - Err if something went wrong
pub fn ensure_singleton() -> Result<bool, Error> {
    // Check if we've already created a singleton in this process
    if SINGLETON_CREATED.load(Ordering::SeqCst) {
        debug!("Singleton already initialized in this process");
        return Ok(true);
    }

    // Class name for our window (must match what we use elsewhere)
    let class_name = CString::new("SyncthingersMainClass").unwrap();
    
    // Check if another instance is already running
    let other_window = unsafe { FindWindowA(class_name.as_ptr(), std::ptr::null()) };
    
    if !other_window.is_null() {
        info!("Another instance is already running, activating it instead");
        
        // Send a message to the other window to bring it to foreground
        let result = unsafe { SendMessageA(other_window, WM_APP_SHOW_WINDOW, 0, 0) };
        if result == 0 {
            error!("Failed to send activation message to existing instance");
            return Err(Error::new(ErrorKind::Other, "Failed to communicate with existing instance"));
        }
        
        // Another instance exists and was notified
        return Ok(false);
    }
    
    // Mark that we've created a singleton
    SINGLETON_CREATED.store(true, Ordering::SeqCst);
    
    // This is the first instance
    info!("This is the first and only instance of the application");
    Ok(true)
}

/// Get the custom message ID for showing the window
pub fn get_show_window_message() -> u32 {
    WM_APP_SHOW_WINDOW
}

/// Creates a window handle that will be used for singleton detection
pub fn create_main_window() -> HWND {
    // This would typically register a window class and create a window
    // For simplicity, we're just returning null for now
    // In a real implementation, we would create a hidden window
    std::ptr::null_mut()
} 