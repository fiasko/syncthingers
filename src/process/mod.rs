use crate::error::{AppError, AppResult};
use crate::config::Config;
use log::{debug, info, error};
use std::process::{Child, Command};

pub struct ProcessManager {
    config: Config,
    syncthing_process: Option<Child>,
}

impl ProcessManager {
    pub fn new(config: Config) -> Self {
        ProcessManager {
            config,
            syncthing_process: None,
        }
    }
    
    pub fn start_syncthing(&mut self) -> AppResult<()> {
        if self.is_running() {
            info!("Syncthing is already running");
            return Ok(());
        }
        
        info!("Starting Syncthing process");
        // This will be implemented in Phase 4
        debug!("Would start Syncthing at path: {}", self.config.syncthing_path);
        
        Ok(())
    }
    
    pub fn stop_syncthing(&mut self) -> AppResult<()> {
        if !self.is_running() {
            debug!("Syncthing is not running");
            return Ok(());
        }
        
        info!("Stopping Syncthing process");
        // This will be implemented in Phase 4
        
        Ok(())
    }
    
    pub fn is_running(&self) -> bool {
        self.syncthing_process.is_some()
    }
    
    pub fn get_status(&self) -> String {
        if self.is_running() {
            "Syncthing is running".to_string()
        } else {
            "Syncthing is not running".to_string()
        }
    }
} 