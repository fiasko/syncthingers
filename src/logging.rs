use crate::app_dirs::AppDirs;
use simplelog::{Config as LogConfig, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;

pub fn init_logging(log_level: LevelFilter, app_dirs: &AppDirs) {
    // Always use AppDirs for log file path and directory management
    app_dirs.ensure_exists().ok();
    let log_path = app_dirs.log_file_path();

    let log_file = match File::create(&log_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create log file at {}: {}", log_path.display(), e);
            return;
        }
    };

    let config = match log_level {
        LevelFilter::Debug => ConfigBuilder::new()
            .set_target_level(LevelFilter::Info)
            .build(),
        _ => LogConfig::default(),
    };

    if let Err(e) = WriteLogger::init(log_level, config, log_file) {
        eprintln!("Failed to initialize logger: {}", e);
    }

    log::info!(
        "Logging initialized at level {} to file: {}",
        log_level,
        log_path.display()
    );
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
