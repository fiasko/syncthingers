#![windows_subsystem = "windows"]

mod singleton;
mod logging;
mod config;
mod process;
mod tray_ui;
mod app_state;
mod error_handling;

use simplelog::LevelFilter;
use config::Config;
use std::env;

fn main() {
    // Check for command line argument to only create default config and exit
    let args: Vec<String> = env::args().collect();
   
    let config = Config::load_or_create("configuration.json")
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to load or create configuration file: {e}");
            std::process::exit(1);
        });

    // Check if --create-config argument is present and exit
    if args.iter().any(|a| a == "--create-config") {
        return;
    }

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
    
    // Create shared app state
    let app_state = std::sync::Arc::new(std::sync::Mutex::new(app_state::AppState::new(config)));

    // Create tray UI
    let mut tray_ui = tray_ui::TrayUi::new(app_state.clone()).unwrap_or_else(|e| {
        log::error!("Failed to create tray UI: {e}");
        error_handling::show_native_error_dialog(&format!("Failed to create tray UI: {e}"), "Syncthingers Error");
        std::process::exit(1);
    });
    tray_ui.setup_tray_menu().unwrap_or_else(|e| {
        log::error!("Failed to set up tray menu: {e}");
        error_handling::show_native_error_dialog(&format!("Failed to set up tray menu: {e}"), "Syncthingers Error");
        std::process::exit(1);
    });

    // Keep the main thread alive so the tray icon stays visible
    log::info!("Tray UI running. Application started.");
    loop {
        std::thread::park();
    }
}
