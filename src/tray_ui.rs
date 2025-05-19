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
        let tray = TrayItem::new("Syncthingers", tray_item::IconSource::Resource("syncthing_green"))?;
        // Detect running Syncthing process using detect_existing
        let state = {
            let state_guard = app_state.lock().unwrap();
            let exe_path = &state_guard.config.syncthing_path;
            if let Ok(Some(_)) = crate::process::SyncthingProcess::detect_existing(exe_path) {
                TrayState::Running
            } else {
                TrayState::Stopped
            }
        };
        Ok(Self {
            tray,
            state,
            app_state,
        })
    }

    pub fn set_state(&mut self, state: TrayState) {
        match state {
            TrayState::Running => {
                let _ = self.tray.set_icon(tray_item::IconSource::Resource("syncthing_green"));
            }
            TrayState::Stopped => {
                let _ = self.tray.set_icon(tray_item::IconSource::Resource("syncthing_red"));
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

    pub fn recreate_tray_menu(&mut self) -> Result<(), AppError> {
        let app_state = self.app_state.clone();
        let start_stop_label = match self.state {
            TrayState::Running => "Stop Syncthing",
            TrayState::Stopped => "Start Syncthing",
        };
        let _ = self.tray.add_menu_item(start_stop_label, move || {
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

    pub fn handle_menu_action(&mut self, action: TrayMenuAction) -> Result<(), AppError> {
        let mut state = self.app_state.lock().unwrap();
        match action {
            TrayMenuAction::StartStop => {
                if state.syncthing_running() {
                    state.stop_syncthing()?;
                    drop(state);
                    let this = self;
                    this.set_state(TrayState::Stopped);
                    this.recreate_tray_menu()?;
                } else {
                    state.start_syncthing()?;
                    drop(state);
                    let this = self;
                    this.set_state(TrayState::Running);
                    this.recreate_tray_menu()?;
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
        }
        Ok(())
    }

    pub fn setup_tray_menu(&mut self) -> Result<(), AppError> {
        self.recreate_tray_menu()
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
        }
        Ok(())
    }
}
