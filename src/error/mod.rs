use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Syncthing process error: {0}")]
    Process(String),
    
    #[error("System tray error: {0}")]
    Tray(String),
    
    #[error("Singleton error: {0}")]
    Singleton(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::Unknown(error)
    }
}

impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::Unknown(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Config(format!("JSON error: {}", error))
    }
} 