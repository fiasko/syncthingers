use log::{info, error, warn, debug};
use simplelog::{Config, LevelFilter, SimpleLogger, WriteLogger, TermLogger, TerminalMode, ColorChoice};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

// Internal modules
mod config;
mod error;
mod singleton;
mod tray;
mod process;

use config::Config as AppConfig;
use error::{AppError, AppResult};

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    let log_dir = PathBuf::from("logs");
    std::fs::create_dir_all(&log_dir)?;
    
    // Setup file logger
    let log_file = log_dir.join("syncthingers.log");
    let file = File::create(log_file)?;
    
    // Set up both file and terminal logging
    let loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![
        WriteLogger::new(LevelFilter::Debug, Config::default(), file),
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
    ];
    
    simplelog::CombinedLogger::init(loggers)?;
    
    info!("Logging initialized");
    Ok(())
}

fn check_singleton() -> AppResult<bool> {
    match singleton::ensure_singleton() {
        Ok(is_first_instance) => {
            if !is_first_instance {
                info!("Another instance is already running, exiting this instance");
                // Ensure logs are flushed before exiting
                log::logger().flush();
            }
            Ok(is_first_instance)
        },
        Err(e) => {
            error!("Failed to check for other instances: {}", e);
            // Ensure logs are flushed
            log::logger().flush();
            Err(AppError::Singleton(format!("Singleton check failed: {}", e)))
        }
    }
}

fn run() -> AppResult<()> {
    // Check singleton first, return early if another instance exists
    if !check_singleton()? {
        return Ok(());
    }
    
    // Only proceed if this is the first instance
    info!("This is the primary instance, continuing startup");
    
    // Load configuration
    let config = AppConfig::load()?;
    
    // TODO: Initialize system tray, process management, etc.
    
    info!("Application running with configuration: {:?}", config);
    
    // Placeholder for main loop
    // In a real implementation, we would have an event loop here
    std::thread::sleep(std::time::Duration::from_secs(10));
    
    Ok(())
}

fn main() {
    // Initialize logging
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        // Continue execution without logging
    }
    
    info!("Syncthingers starting up");
    
    // Run the application
    if let Err(e) = run() {
        error!("Application error: {}", e);
        eprintln!("Application error: {}", e);
    }
    
    info!("Syncthingers shutting down");
    // Final log flush before exit
    log::logger().flush();
}
