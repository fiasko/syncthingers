use tray_item::TrayItem;
use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::app_state::AppState;
use crate::error_handling::AppError;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrayState {
    Running,
    Stopped,
}

#[derive(Debug)]
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
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Result<Arc<Mutex<Self>>, Box<dyn Error>> {
        let tray = TrayItem::new("Syncthingers", tray_item::IconSource::Resource("syncthing_red"))?;
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
        let tray_ui = Self {
            tray,
            state,
            app_state: app_state.clone(),
        };
        // Spawn a background thread to monitor Syncthing process state and update tray UI
        let tray_ui_ptr = Arc::new(Mutex::new(tray_ui));
        let tray_ui_weak = Arc::downgrade(&tray_ui_ptr);
        let app_state = app_state.clone();
        std::thread::spawn(move || {
            let mut last_state = None;
            loop {
                let running = {
                    let mut state = app_state.lock().unwrap();
                    state.syncthing_running()
                };
                // Determine process state: external, started by app, or not running
                let (new_state, process_origin) = {
                    let mut state = app_state.lock().unwrap();
                    if let Some(proc) = &mut state.syncthing_process {
                        if proc.started_by_app {
                            (TrayState::Running, "started by app")
                        } else {
                            (TrayState::Running, "external")
                        }
                    } else {
                        (TrayState::Stopped, "not running")
                    }
                };
                if last_state.as_ref() != Some(&new_state) {
                    match process_origin {
                        "started by app" => log::info!("Syncthing process state: running (started by this app)"),
                        "external" => log::info!("Syncthing process state: running (external)"),
                        "not running" => log::info!("Syncthing process state: not running"),
                        _ => {}
                    }
                    if let Some(tray_ui_arc) = tray_ui_weak.upgrade() {
                        let mut tray_ui = tray_ui_arc.lock().unwrap();
                        tray_ui.set_state(new_state);
                        let _ = tray_ui.recreate_tray_menu();
                    }
                    last_state = Some(new_state);
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
        Ok(tray_ui_ptr)
    }

    pub fn set_state(&mut self, state: TrayState) {
        self.state = state;
    }

    pub fn recreate_tray_menu(&mut self) -> Result<(), AppError> {
        let icon = match self.state {
            TrayState::Running => tray_item::IconSource::Resource("syncthing_green"),
            TrayState::Stopped => tray_item::IconSource::Resource("syncthing_red"),
        };
        let mut new_tray = TrayItem::new("Syncthingers", icon)
            .map_err(|e| AppError::TrayUiError(format!("Failed to recreate tray: {e}")))?;
        let app_state = self.app_state.clone();
        let start_stop_label = match self.state {
            TrayState::Running => "Stop Syncthing",
            TrayState::Stopped => "Start Syncthing",
        };
        let _ = new_tray.add_menu_item(start_stop_label, move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::StartStop);
        });
        let app_state = self.app_state.clone();
        let _ = new_tray.add_menu_item("Open Syncthing Web UI", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::OpenWebUI);
        });
        let app_state = self.app_state.clone();
        let _ = new_tray.add_menu_item("Open Configuration", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::OpenConfig);
        });
        let app_state = self.app_state.clone();
        let _ = new_tray.add_menu_item("Exit", move || {
            let _ = Self::handle_menu_action_static(app_state.clone(), TrayMenuAction::Exit);
        });
        self.tray = new_tray;
        Ok(())
    }
 
    pub fn setup_tray_menu(&mut self) -> Result<(), AppError> {
        self.recreate_tray_menu()
    }

    pub fn handle_menu_action_static(app_state: Arc<Mutex<AppState>>, action: TrayMenuAction) -> Result<(), AppError> {
        log::info!("Tray menu action: {:?}", action);
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
