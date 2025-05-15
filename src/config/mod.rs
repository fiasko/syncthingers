use crate::error::{AppError, AppResult};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Main configuration structure for the application
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Path to the Syncthing executable
    pub syncthing_path: String,
    
    /// URL for Syncthing web UI
    pub web_ui_url: String,
    
    /// Additional arguments for Syncthing
    pub syncthing_args: Vec<String>,
    
    /// Log configuration
    pub logging: LogConfig,
    
    /// UI configuration
    pub ui: UiConfig,
}

/// Logging configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Path to log file
    pub file_path: PathBuf,
    
    /// Maximum log file size in KB
    pub max_size_kb: u64,
    
    /// Maximum number of log files to keep
    pub max_files: u32,
}

/// UI configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct UiConfig {
    /// Whether to show notifications
    pub show_notifications: bool,
    
    /// Whether to minimize to tray on startup
    pub minimize_to_tray: bool,
    
    /// Whether to start Syncthing on app startup
    pub start_syncthing_on_startup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            syncthing_path: "syncthing.exe".to_string(),
            web_ui_url: "http://127.0.0.1:8384".to_string(),
            syncthing_args: vec![],
            logging: LogConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "info".to_string(),
            file_path: PathBuf::from("logs/syncthingers.log"),
            max_size_kb: 1024, // 1MB
            max_files: 3,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            show_notifications: true,
            minimize_to_tray: true,
            start_syncthing_on_startup: true,
        }
    }
}

impl Config {
    /// Load configuration from file, creating default if not exists
    pub fn load() -> AppResult<Self> {
        let config_path = get_config_path();
        
        if !config_path.exists() {
            debug!("Configuration file not found, creating default");
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }
        
        debug!("Loading configuration from {:?}", config_path);
        let mut file = File::open(&config_path)
            .map_err(|e| AppError::Config(format!("Failed to open config file: {}", e)))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;
            
        let config: Config = serde_json::from_str(&contents)?;
        info!("Configuration loaded successfully");
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> AppResult<()> {
        let config_path = get_config_path();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| AppError::Config(format!("Failed to create config directory: {}", e)))?;
            }
        }
        
        debug!("Saving configuration to {:?}", config_path);
        let json = serde_json::to_string_pretty(self)?;
        
        let mut file = File::create(&config_path)
            .map_err(|e| AppError::Config(format!("Failed to create config file: {}", e)))?;
            
        file.write_all(json.as_bytes())
            .map_err(|e| AppError::Config(format!("Failed to write config file: {}", e)))?;
            
        info!("Configuration saved successfully");
        Ok(())
    }
}

/// Get the path to the configuration file
pub fn get_config_path() -> PathBuf {
    let mut path = if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("Syncthingers")
    } else {
        PathBuf::from(".")
    };
    
    path.push("configuration.json");
    path
} 