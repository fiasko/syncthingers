use tray_item::TrayItem;
use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::app_state::AppState;
use crate::error_handling::AppError;

pub enum TrayState {
    Running,
    Stopped,
}

pub enum TrayMenuAction {
    StartStop,
    OpenWebUI,
    OpenConfig,
    Exit,
}

pub struct TrayUi {
    tray: TrayItem,
    state: TrayState,
    app_state: Arc<Mutex<AppState>>,
}

impl TrayUi {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Result<Self, Box<dyn Error>> {
        let tray = TrayItem::new("Syncthingers", tray_item::IconSource::Resource("assets/icons/syncthing_red.ico"))?;
        Ok(Self {
            tray,
            state: TrayState::Stopped,
            app_state,
        })
    }

    pub fn set_state(&mut self, state: TrayState) {
        match state {
            TrayState::Running => {
                let _ = self.tray.set_icon(tray_item::IconSource::Resource("assets/icons/syncthing_green.ico"));
            }
            TrayState::Stopped => {
                let _ = self.tray.set_icon(tray_item::IconSource::Resource("assets/icons/syncthing_red.ico"));
            }
        }
        self.state = state;
    }

    pub fn add_menu<F>(&mut self, start_stop: F, open_web: F, open_config: F, exit: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let start_stop_label = match self.state {
            TrayState::Running => "Stop Syncthing",
            TrayState::Stopped => "Start Syncthing",
        };
        let _ = self.tray.add_menu_item(start_stop_label, start_stop);
        let _ = self.tray.add_menu_item("Open Syncthing Web UI", open_web);
        let _ = self.tray.add_menu_item("Open App Configuration", open_config);
        let _ = self.tray.add_menu_item("Exit", exit);
    }

    pub fn handle_menu_action(&self, action: TrayMenuAction) -> Result<(), AppError> {
        let mut state = self.app_state.lock().unwrap();
        match action {
            TrayMenuAction::StartStop => {
                if state.syncthing_running() {
                    state.stop_syncthing()?;
                } else {
                    state.start_syncthing()?;
                }
                // Update tray icon and menu here if needed
            }
            TrayMenuAction::OpenWebUI => {
                opener::open(&state.config.web_ui_url).map_err(|e| AppError::TrayUiError(format!("Failed to open web UI: {}", e)))?;
            }
            TrayMenuAction::OpenConfig => {
                // Open config file in editor
                opener::open("configuration.json").map_err(|e| AppError::TrayUiError(format!("Failed to open config: {}", e)))?;
            }
            TrayMenuAction::Exit => {
                state.stop_syncthing().ok();
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn setup_tray_menu(&mut self) -> Result<(), AppError> {
        let app_state = self.app_state.clone();
        let _ = self.tray.add_menu_item("Start/Stop Syncthing", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::StartStop);
        });
        let app_state = self.app_state.clone();
        let _ = self.tray.add_menu_item("Open Syncthing Web UI", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::OpenWebUI);
        });
        let app_state = self.app_state.clone();
        let _ = self.tray.add_menu_item("Open Configuration", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::OpenConfig);
        });
        let app_state = self.app_state.clone();
        let _ = self.tray.add_menu_item("Exit", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::Exit);
        });
        Ok(())
    }

    pub fn handle_menu_action_static(app_state: Arc<Mutex<AppState>>, action: TrayMenuAction) -> Result<(), AppError> {
        let mut state = app_state.lock().unwrap();
        match action {
            TrayMenuAction::StartStop => {
                if state.syncthing_running() {
                    state.stop_syncthing()?;
                } else {
                    state.start_syncthing()?;
                }
            }
            TrayMenuAction::OpenWebUI => {
                opener::open(&state.config.web_ui_url).map_err(|e| AppError::TrayUiError(format!("Failed to open web UI: {}", e)))?;
            }
            TrayMenuAction::OpenConfig => {
                opener::open("configuration.json").map_err(|e| AppError::TrayUiError(format!("Failed to open config: {}", e)))?;
            }
            TrayMenuAction::Exit => {
                state.stop_syncthing().ok();
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(())
    }
}
