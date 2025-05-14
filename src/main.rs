#![windows_subsystem = "windows"] // Hide console window on Windows release builds

mod single_instance;
mod logger; // Declare the logger module

fn main() {
    // Initialize logging as early as possible.
    if let Err(e) = logger::init_logging() {
        eprintln!("Fatal Error: Failed to initialize logging: {}. Exiting.", e);
        // In a real GUI app, you might show a message box here.
        // For now, if logging fails, we might not want to proceed.
        return;
    }

    // Ensure only one instance of the application is running.
    // This should be one of the very first things the application does.
    if let Err(e) = single_instance::ensure_single_instance() {
        log::error!("Failed to ensure single instance: {}. Exiting.", e);
        // Optionally, show a message box to the user on Windows.
        return; // Exit if another instance is running or if there was an error.
    }

    log::info!("Syncthingers application starting up..."); // Example log message
    // ... rest of your application logic will go here ...
    // For now, let's just keep the app alive for a bit to see logs, then exit.
    // In a real tray app, this would be an event loop.
    // std::thread::sleep(std::time::Duration::from_secs(5));
    // log::info!("Syncthingers shutting down.");
}