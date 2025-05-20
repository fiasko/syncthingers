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
        // Try to detect and attach to an external Syncthing process
        let syncthing_process = match SyncthingProcess::detect_external(&config.syncthing_path) {
            Ok(Some(proc)) => {
                log::info!("Attached to external Syncthing process.");
                Some(proc)
            },
            Ok(None) => None,
            Err(e) => {
                log::warn!("Failed to detect external Syncthing process: {}", e);
                None
            }
        };
        Self {
            config,
            syncthing_process,
            tray_ui: None,
        }
    }

    /// Attempts to detect and attach to an external Syncthing process, updating state.
    pub fn detect_and_attach_external(&mut self) -> Result<bool, crate::error_handling::AppError> {
        match SyncthingProcess::detect_external(&self.config.syncthing_path) {
            Ok(Some(proc)) => {
                self.syncthing_process = Some(proc);
                log::info!("Attached to external Syncthing process.");
                Ok(true)
            },
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("Failed to detect external Syncthing process: {}", e);
                Err(crate::error_handling::AppError::ProcessError(format!("Failed to detect external Syncthing: {}", e)))
            }
        }
    }

    pub fn syncthing_running(&mut self) -> bool {
        // Always check for an external process if not running
        if self.syncthing_process.is_none() {
            if let Err(e) = self.detect_and_attach_external() {
                log::warn!("Error during external Syncthing detection: {}", e);
            }
        }
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
