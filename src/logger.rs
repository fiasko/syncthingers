use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::path::PathBuf;

const LOG_FILE_NAME: &str = "syncthingers.log";

/// Initializes the logging system.
///
/// Attempts to set up:
/// 1. A file logger (e.g., `syncthingers.log` next to the executable).
/// 2. A terminal logger (for debug builds, visible if run from a console).
///
/// Returns `Ok` on success, or an `Err` string describing the failure.
pub fn init_logging() -> Result<(), String> {
    let mut log_path = match std::env::current_exe() {
        Ok(mut exe_path) => {
            exe_path.pop(); // Remove the executable name to get the parent directory
            exe_path
        }
        Err(e) => {
            // Fallback to current directory if we can't determine exe path
            eprintln!("Warning: Could not determine executable path for logging: {}. Using current directory.", e);
            PathBuf::from(".")
        }
    };
    log_path.push(LOG_FILE_NAME);

    let log_file = File::create(&log_path)
        .map_err(|e| format!("Failed to create log file at {:?}: {}", log_path, e))?;

    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();

    // File logger
    loggers.push(WriteLogger::new(
        LevelFilter::Info, // Default log level for file
        ConfigBuilder::new().set_time_format_rfc3339().build(), // Add timestamps
        log_file,
    ));

    // Terminal logger for debug builds
    if cfg!(debug_assertions) {
        loggers.push(TermLogger::new(
            LevelFilter::Debug, // More verbose for console in debug
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    }

    CombinedLogger::init(loggers).map_err(|e| format!("Failed to initialize logger: {}", e))
}