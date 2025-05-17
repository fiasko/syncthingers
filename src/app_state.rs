use crate::config::Config;
use crate::process::SyncthingProcess;
use crate::tray_ui::TrayUi;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Config,
    pub syncthing_process: Option<SyncthingProcess>,
    pub tray_ui: Option<TrayUi>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            syncthing_process: None,
            tray_ui: None,
        }
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;
