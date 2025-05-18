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

    pub fn syncthing_running(&mut self) -> bool {
        if let Some(proc) = &mut self.syncthing_process {
            proc.is_running()
        } else {
            false
        }
    }

    pub fn start_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if self.syncthing_running() {
            return Ok(());
        }
        let exe_path = &self.config.syncthing_path;
        let args = &self.config.startup_args;
        let proc = crate::process::SyncthingProcess::start(exe_path, args)
            .map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to start Syncthing: {}", e)))?;
        self.syncthing_process = Some(proc);
        log::info!("Syncthing process started successfully.");
        Ok(())
    }

    pub fn stop_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(proc) = &mut self.syncthing_process {
            proc.stop().map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to stop Syncthing: {}", e)))?;
            log::info!("Syncthing process stopped successfully.");
        }
        self.syncthing_process = None;
        Ok(())
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;
