#![windows_subsystem = "windows"]

mod app_dirs;
mod app_state;
pub mod config;
mod error_handling;
mod logging;
mod process;
mod process_monitor;
mod singleton;
mod tray_ui;

use config::Config;
use simplelog::LevelFilter;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect(); // Parse command line arguments
    let mut log_level = LevelFilter::Error;
    let mut app_dirs_override = None;
    for arg in &args {
        if let Some(lvl) = arg.strip_prefix("--log-level=") {
            log_level = logging::log_level_from_str(lvl);
        } else if arg == "--portable" {
            // In portable mode, use the current working directory
            if let Ok(current_dir) = std::env::current_dir() {
                app_dirs_override = Some(current_dir);
            } else {
                eprintln!("Error: failed to get current working directory for portable mode");
                std::process::exit(1);
            }
        }
    }

    // Create the AppDirs instance (stateful)
    let app_dirs = match app_dirs::AppDirs::new(app_dirs_override) {
        Ok(ad) => ad,
        Err(e) => {
            eprintln!("Error: failed to determine application directory: {e}");
            std::process::exit(1);
        }
    };

    // Ensure the directory exists
    if let Err(e) = app_dirs.ensure_exists() {
        eprintln!("Error: failed to create application directories: {e}");
        std::process::exit(1);
    }

    // Get log file path
    let log_file_path = app_dirs.log_file_path();
    logging::init_logging(log_level, &log_file_path);

    // Get config file path
    let config_file_path = app_dirs.config_file_path();
    log::debug!("Using configuration file: {}", config_file_path.display());

    let config = Config::load_or_create(config_file_path).unwrap_or_else(|e| {
        eprintln!("Error: failed to load or create configuration file: {e}");
        std::process::exit(1);
    });

    // Check if --create-config argument is present and exit
    if args.iter().any(|a| a == "--create-config") {
        return;
    }

    // Reconfigure logging as per config
    let config_log_level = logging::log_level_from_str(&config.log_level);
    logging::set_log_level(config_log_level);

    log::info!("Startup arguments: {:?}", config.startup_args);

    if singleton::platform::SingletonGuard::acquire().is_none() {
        log::warn!("Another instance of the application is already running. Exiting.");
        return;
    }

    log::info!("Application starting");

    // Create shared app state
    let app_state = std::sync::Arc::new(std::sync::Mutex::new(app_state::AppState::new(config, app_dirs.clone())));

    // Create tray UI
    let tray_ui = tray_ui::TrayUi::new(app_state.clone()).unwrap_or_else(|e| {
        log::error!("Failed to create tray UI: {e}");
        error_handling::show_native_error_dialog(
            &format!("Failed to create tray UI: {e}"),
            "Syncthingers Error",
        );
        std::process::exit(1);
    });
    {
        let mut tray_ui_guard = tray_ui.lock().unwrap();
        tray_ui_guard.setup_tray_menu().unwrap_or_else(|e| {
            log::error!("Failed to set up tray menu: {e}");
            error_handling::show_native_error_dialog(
                &format!("Failed to set up tray menu: {e}"),
                "Syncthingers Error",
            );
            std::process::exit(1);
        });
    }

    // Keep the main thread alive so the tray icon stays visible
    log::info!("Tray UI running. Application started.");
    loop {
        std::thread::park();
    }
}
