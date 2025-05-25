use simplelog::{Config as LogConfig, LevelFilter, WriteLogger, ConfigBuilder};
use std::fs::File;
use std::path::Path;

pub fn init_logging(log_level: LevelFilter, log_path: impl AsRef<Path>) {
    let path = log_path.as_ref();
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory: {}", e);
            }
        }
    }
    
    let log_file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create log file at {}: {}", path.display(), e);
            return;
        }
    };

     let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Info)
        .build();
    
    if let Err(e) = WriteLogger::init(log_level, config, log_file) {
        eprintln!("Failed to initialize logger: {}", e);
    }
    
    log::info!("Logging initialized at level {} to file: {}", 
              log_level, path.display());
}

pub fn set_log_level(level: LevelFilter) {
    log::set_max_level(level);
}

pub fn log_level_from_str(level: &str) -> LevelFilter {
    match level.to_lowercase().as_str() {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        _ => LevelFilter::Info,
    }
}
