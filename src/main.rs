#![windows_subsystem = "windows"]

mod singleton;
mod logging;
pub mod config;
mod process;
mod tray_ui;
mod app_state;
mod error_handling;
mod app_dirs;

use simplelog::LevelFilter;
use config::Config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();    // Parse command line arguments
    let mut log_level = LevelFilter::Error;
    let mut config_path_override = None;
    
    for arg in &args {
        if let Some(lvl) = arg.strip_prefix("--log-level=") {
            log_level = logging::log_level_from_str(lvl);
        } else if let Some(path) = arg.strip_prefix("--config-path=") {
            config_path_override = Some(std::path::PathBuf::from(path));
        }
    }
    
    // Ensure application directories exist
    if let Err(e) = app_dirs::ensure_app_dirs_exist() {
        eprintln!("Error: failed to create application directories: {e}");
        std::process::exit(1);
    }
    
    // Try to migrate configuration from exe directory if needed
    let _ = app_dirs::migrate_from_exe_dir();
      // Get log file path
    let log_file_path = app_dirs::get_log_file_path(config_path_override.as_deref())
        .unwrap_or_else(|| std::path::PathBuf::from("syncthingers.log"));
        
    logging::init_logging(log_level, &log_file_path);
    
    // Get config file path
    let config_file_path = app_dirs::get_config_file_path(config_path_override.as_deref())
        .unwrap_or_else(|| std::path::PathBuf::from("configuration.json"));
    
    log::info!("Using configuration file: {}", config_file_path.display());
    
    let config = Config::load_or_create(config_file_path)
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to load or create configuration file: {e}");
            std::process::exit(1);
        });

    // Check if --create-config argument is present and exit
    if args.iter().any(|a| a == "--create-config") {
        return;
    }

    // Reconfigre logging as per config
    let config_log_level = logging::log_level_from_str(&config.log_level);
    logging::set_log_level(config_log_level);
    
    log::info!("Startup arguments: {:?}", config.startup_args);

    if singleton::platform::SingletonGuard::acquire().is_none() {
        log::warn!("Another instance of the application is already running. Exiting.");
        return;
    }
    log::info!("Application starting");
    
    // Create shared app state
    let app_state = std::sync::Arc::new(std::sync::Mutex::new(app_state::AppState::new(config)));

    // Create tray UI
    let tray_ui = tray_ui::TrayUi::new(app_state.clone()).unwrap_or_else(|e| {
        log::error!("Failed to create tray UI: {e}");
        error_handling::show_native_error_dialog(&format!("Failed to create tray UI: {e}"), "Syncthingers Error");
        std::process::exit(1);
    });
    {
        let mut tray_ui_guard = tray_ui.lock().unwrap();
        tray_ui_guard.setup_tray_menu().unwrap_or_else(|e| {
            log::error!("Failed to set up tray menu: {e}");
            error_handling::show_native_error_dialog(&format!("Failed to set up tray menu: {e}"), "Syncthingers Error");
            std::process::exit(1);
        });
    }

    // Keep the main thread alive so the tray icon stays visible
    log::info!("Tray UI running. Application started.");
    loop {
        std::thread::park();
    }
}
