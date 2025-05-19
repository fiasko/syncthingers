use simplelog::{Config as LogConfig, LevelFilter, WriteLogger};
use std::fs::File;
use std::path::Path;

pub fn init_logging(log_level: LevelFilter, log_path: &str) {
    let log_file = File::create(Path::new(log_path)).expect("Failed to create log file");
    WriteLogger::init(log_level, LogConfig::default(), log_file).expect("Failed to initialize logger");
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
