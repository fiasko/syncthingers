#![windows_subsystem = "windows"]

mod singleton;
mod logging;

use simplelog::LevelFilter;

fn main() {
    // Initialize logging (for now, hardcoded; later, use config)
    logging::init_logging(LevelFilter::Info, "syncthingers.log");

    // Singleton enforcement using portable interface
    if singleton::platform::SingletonGuard::acquire().is_none() {
        log::error!("Another instance detected. Exiting.");
        std::process::exit(0);
    }

    log::info!("Application starting");
}
