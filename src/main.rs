#![windows_subsystem = "windows"]

mod singleton;
mod logging;
mod config;

use simplelog::LevelFilter;
use config::Config;

fn main() {
    let config = Config::load_or_create("configuration.json")
        .expect("Failed to load or create configuration file");

    // Initialize logging as per config
    let log_level = match config.log_level.to_lowercase().as_str() {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };
    logging::init_logging(log_level, "syncthingers.log");
    
    if singleton::platform::SingletonGuard::acquire().is_none() {
        log::warn!("Another instance of the application is already running. Exiting.");
        return;
    }
    log::info!("Application starting");
    println!("Hello, world!");
}
