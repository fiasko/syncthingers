use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Process error: {0}")]
    ProcessError(String),
    #[error("Tray UI error: {0}")]
    TrayUiError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Unknown(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

// Helper for showing native error dialogs on Windows
#[cfg(target_os = "windows")]
pub fn show_native_error_dialog(msg: &str, caption: &str) {
    use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};
    use std::ptr;
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
