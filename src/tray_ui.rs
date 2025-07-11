use log::{debug, info, warn};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tray_item::TrayItem;

use crate::app_state::AppState;
use crate::config::Config;
use crate::error_handling::AppError;

/// Represents the current state of the system tray UI.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrayState {
    Running,
    Stopped,
}

/// Actions that can be triggered from the system tray menu.
#[derive(Debug, Copy, Clone)]
pub enum TrayMenuAction {
    StartStop,
    OpenWebUI,
    OpenConfig,
    Exit,
}

/// System tray UI component for Syncthingers application.
pub struct TrayUi {
    tray: TrayItem,
    state: TrayState,
    app_state: Arc<Mutex<AppState>>,
}

impl TrayUi {
    /// Creates a new TrayUi instance with the given application state.
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Result<Arc<Mutex<Self>>, Box<dyn Error>> {
        // Initialize tray with initial icon state
        let tray = TrayItem::new(
            "Syncthingers",
            tray_item::IconSource::Resource("syncthing_red"),
        )?;

        // Determine initial state by detecting running Syncthing process
        let initial_state = Self::detect_initial_state(&app_state)?;

        let tray_ui = Self {
            tray,
            state: initial_state,
            app_state: app_state.clone(),
        };

        // Create thread-safe reference to tray UI
        let tray_ui_ptr = Arc::new(Mutex::new(tray_ui));

        // Start monitoring thread
        Self::start_monitoring_thread(tray_ui_ptr.clone(), app_state)?;

        Ok(tray_ui_ptr)
    }

    /// Detects the initial state of Syncthing (running or stopped).
    fn detect_initial_state(app_state: &Arc<Mutex<AppState>>) -> Result<TrayState, Box<dyn Error>> {
        let mut state_guard = app_state.lock().map_err(|_| "Failed to lock app state")?;

        if state_guard.syncthing_running()
            || state_guard.detect_and_attach_external().unwrap_or(false)
        {
            Ok(TrayState::Running)
        } else {
            Ok(TrayState::Stopped)
        }
    }

    /// Starts a background thread to monitor Syncthing process state and update tray UI.
    fn start_monitoring_thread(
        tray_ui_ptr: Arc<Mutex<Self>>,
        app_state: Arc<Mutex<AppState>>,
    ) -> Result<(), Box<dyn Error>> {
        // Create a weak reference to avoid circular references
        let tray_ui_weak = Arc::downgrade(&tray_ui_ptr);

        // Spawn a polling thread
        thread::spawn(move || {
            // Get initial process state
            let initial_state = Self::get_current_process_state(&app_state);
            if let Some(tray_ui_arc) = tray_ui_weak.upgrade() {
                if let Ok(mut tray_ui) = tray_ui_arc.lock() {
                    tray_ui.set_state(initial_state.0);
                    if let Err(e) = tray_ui.recreate_tray_menu() {
                        warn!("Failed to recreate tray menu: {}", e);
                    }
                }
            }

            // Initialize state tracking variable
            let mut last_state = Some(initial_state.0);

            // Simple polling loop
            loop {
                // Poll every 2 seconds
                thread::sleep(Duration::from_secs(2));

                // Check current process state
                let new_state = Self::get_current_process_state(&app_state);

                // Update UI if state changed
                if last_state.as_ref() != Some(&new_state.0) {
                    Self::log_process_state(&new_state.1);
                    debug!("State change detected by polling: {:?}", new_state.0);

                    if let Some(tray_ui_arc) = tray_ui_weak.upgrade() {
                        if let Ok(mut tray_ui) = tray_ui_arc.lock() {
                            tray_ui.set_state(new_state.0);
                            if let Err(e) = tray_ui.recreate_tray_menu() {
                                warn!("Failed to recreate tray menu: {}", e);
                            }
                        }
                    } else {
                        // TrayUi was dropped, exit the monitoring thread
                        debug!("TrayUi was dropped, exiting monitor thread");
                        break;
                    }

                    last_state = Some(new_state.0);
                }
            }
        });

        Ok(())
    }

    /// Gets the current Syncthing process state and its origin.
    fn get_current_process_state(app_state: &Arc<Mutex<AppState>>) -> (TrayState, String) {
        match app_state.lock() {
            Ok(mut state) => {
                // Use syncthing_running() to get the actual state, which handles process cleanup
                if state.syncthing_running() {
                    if let Some(proc) = &state.syncthing_process {
                        if proc.started_by_app {
                            (TrayState::Running, "started by app".to_string())
                        } else {
                            (TrayState::Running, "external".to_string())
                        }
                    } else {
                        // This shouldn't happen, but handle it gracefully
                        (TrayState::Stopped, "not running".to_string())
                    }
                } else {
                    (TrayState::Stopped, "not running".to_string())
                }
            }
            Err(_) => (TrayState::Stopped, "error checking state".to_string()),
        }
    }

    /// Logs the process state change.
    fn log_process_state(process_origin: &str) {
        match process_origin {
            "started by app" => info!("Syncthing process state: running (started by this app)"),
            "external" => info!("Syncthing process state: running (external)"),
            "not running" => info!("Syncthing process state: not running"),
            _ => warn!("Unknown Syncthing process state: {}", process_origin),
        }
    }

    /// Updates the tray state.
    pub fn set_state(&mut self, state: TrayState) {
        self.state = state;
    }

    /// Recreates the tray menu with updated state.
    pub fn recreate_tray_menu(&mut self) -> Result<(), AppError> {
        // Determine icon based on current state
        let icon = match self.state {
            TrayState::Running => tray_item::IconSource::Resource("syncthing_green"),
            TrayState::Stopped => tray_item::IconSource::Resource("syncthing_red"),
        };

        // Create new tray with updated icon
        let mut new_tray = TrayItem::new("Syncthingers", icon)
            .map_err(|e| AppError::TrayUi(format!("Failed to recreate tray: {e}")))?;

        // Add menu items with appropriate callbacks
        self.add_menu_items(&mut new_tray)?;

        // Replace the old tray with new one
        self.tray = new_tray;

        Ok(())
    }

    /// Adds all menu items to the tray.
    fn add_menu_items(&self, tray: &mut TrayItem) -> Result<(), AppError> {
        let start_stop_label = match self.state {
            TrayState::Running => "Stop Syncthing",
            TrayState::Stopped => "Start Syncthing",
        };

        // Start/Stop menu item
        self.add_menu_item(tray, start_stop_label, TrayMenuAction::StartStop)?;

        // Open Web UI menu item
        self.add_menu_item(tray, "Open Syncthing Web UI", TrayMenuAction::OpenWebUI)?;

        // Open Configuration menu item
        self.add_menu_item(tray, "Open Configuration", TrayMenuAction::OpenConfig)?;

        // Exit menu item
        self.add_menu_item(tray, "Exit", TrayMenuAction::Exit)?;

        Ok(())
    }

    /// Helper to add an individual menu item with appropriate callback.
    fn add_menu_item(
        &self,
        tray: &mut TrayItem,
        label: &str,
        action: TrayMenuAction,
    ) -> Result<(), AppError> {
        let app_state = self.app_state.clone();
        let action_clone = action;

        tray.add_menu_item(label, move || {
            _ = Self::handle_menu_action_static(app_state.clone(), action_clone);
        })
        .map_err(|e| AppError::TrayUi(format!("Failed to add menu item '{}': {}", label, e)))
    }

    /// Sets up the initial tray menu.
    pub fn setup_tray_menu(&mut self) -> Result<(), AppError> {
        self.recreate_tray_menu()
    }

    /// Static handler for menu actions.
    pub fn handle_menu_action_static(
        app_state: Arc<Mutex<AppState>>,
        action: TrayMenuAction,
    ) -> Result<(), AppError> {
        info!("Tray menu action: {:?}", action);

        match app_state.lock() {
            Ok(mut state) => Self::process_menu_action(&mut state, action),
            Err(_) => Err(AppError::TrayUi("Failed to lock app state".to_string())),
        }
    }

    /// Processes a menu action with the given application state.
    fn process_menu_action(state: &mut AppState, action: TrayMenuAction) -> Result<(), AppError> {
        match action {
            TrayMenuAction::StartStop => {
                if state.syncthing_running() {
                    state.stop_syncthing()?;
                } else {
                    state.start_syncthing()?;
                }
            }
            TrayMenuAction::OpenWebUI => {
                opener::open(&state.config.web_ui_url)
                    .map_err(|e| AppError::TrayUi(format!("Failed to open web UI: {}", e)))?;
            }
            TrayMenuAction::OpenConfig => {
                // Use the stateful AppDirs instance
                let config_file_path = state.app_dirs.config_file_path();
                log::info!(
                    "Opening configuration file at: {}",
                    config_file_path.display()
                );
                // Check if it exists
                if !config_file_path.exists() {
                    log::error!(
                        "Configuration file does not exist at {}",
                        config_file_path.display()
                    );
                    // If we got here, we couldn't find any config file
                    return Err(AppError::TrayUi("Configuration file not found".to_string()));
                }
                // Open the configuration file
                Config::open_in_editor(&config_file_path)
                    .map_err(|e| AppError::TrayUi(format!("Failed to open config file: {}", e)))?;
            }
            TrayMenuAction::Exit => {
                // Handle process closure based on configuration
                if let Err(e) = state.handle_exit_closure() {
                    log::warn!("Error during exit closure: {}", e);
                }
                std::process::exit(0);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_dirs::AppDirs;

    // Helper for creating test config
    fn create_test_config() -> Config {
        Config {
            syncthing_path: "nonexistent_test_syncthing.exe".to_string(), // Use test pattern
            startup_args: vec![],
            ..Config::default()
        }
    }

    fn dummy_app_dirs() -> AppDirs {
        AppDirs::new(None).unwrap()
    }

    #[test]
    fn test_detect_initial_state_none() {
        // This test would need mock process detection to be fully testable
        // For now, just test the API pattern
        let config = create_test_config();
        let app_dirs = dummy_app_dirs();
        let app_state = Arc::new(Mutex::new(AppState::new(config, app_dirs)));

        // In a real test with mocks, we'd control the process detection result
        let state = TrayUi::detect_initial_state(&app_state);
        assert!(state.is_ok(), "Initial state detection should succeed");
    }

    #[test]
    fn test_process_menu_action_exit() {
        // This test would need to mock std::process::exit for full testing
        // Here we're just validating the API structure
        let config = create_test_config();
        let app_dirs = dummy_app_dirs();
        let _app_state = AppState::new(config, app_dirs);

        // We can't actually test Exit since it calls process::exit
        // but we can ensure the code path doesn't throw exceptions
        // In a real test, we'd mock std::process::exit
    }

    #[test]
    fn test_get_current_process_state() {
        let config = create_test_config();
        let app_dirs = dummy_app_dirs();
        let app_state = Arc::new(Mutex::new(AppState::new(config, app_dirs)));

        // Test with no process running
        let (state, origin) = TrayUi::get_current_process_state(&app_state);
        assert_eq!(state, TrayState::Stopped);
        assert_eq!(origin, "not running");

        // In a real test with mocks, we could test the other states as well
    }
}
