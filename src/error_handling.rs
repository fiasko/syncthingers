use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Tray UI error: {0}")]
    TrayUi(String),
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Config(e.to_string())
    }
}

// Helper for showing native error dialogs on Windows
#[cfg(target_os = "windows")]
pub fn show_native_error_dialog(msg: &str, caption: &str) {
    use std::ptr;
    use winapi::um::winuser::{MB_ICONERROR, MB_OK, MessageBoxW};
    let msg_w: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    let caption_w: Vec<u16> = caption.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            msg_w.as_ptr(),
            caption_w.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

// Dummy alternative for non-Windows platforms: just log the error
#[cfg(not(target_os = "windows"))]
pub fn show_native_error_dialog(msg: &str, caption: &str) {
    eprintln!("{}: {}", caption, msg);
}
