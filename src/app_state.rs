use crate::config::Config;
use crate::process::SyncthingProcess;

/// Represents the application state for Syncthingers.
/// 
/// This struct manages the Syncthing process and configuration,
/// handling both processes started by the app and external processes.
pub struct AppState {
    /// The application configuration
    pub config: Config,
    /// The current Syncthing process being managed, if any
    pub syncthing_process: Option<SyncthingProcess>,
}

impl AppState {
    /// Creates a new AppState with the given configuration.
    /// 
    /// This will attempt to detect and attach to any existing external Syncthing process.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The application configuration
    pub fn new(config: Config) -> Self {
        // Try to detect and attach to an external Syncthing process
        let syncthing_process = match SyncthingProcess::detect_process(&config.syncthing_path, true) {
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
        }
    }

    /// Attempts to detect and attach to an external Syncthing process, updating state.
    pub fn detect_and_attach_external(&mut self) -> Result<bool, crate::error_handling::AppError> {
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true) {
            Ok(Some(proc)) => {
                self.syncthing_process = Some(proc);
                log::info!("Attached to external Syncthing process.");
                Ok(true)
            },
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("Failed to detect external Syncthing process: {}", e);
                Err(crate::error_handling::AppError::ProcessError(format!("Failed to detect external Syncthing: {}", e)))
            }        }
    }
    
    /// Checks if Syncthing is currently running.
    /// 
    /// If no process is currently tracked, this will attempt to detect an external process first.
    /// 
    /// # Returns
    /// 
    /// `true` if Syncthing is running, `false` otherwise
    pub fn syncthing_running(&mut self) -> bool {
        // Always check for an external process if not running
        if self.syncthing_process.is_none() {
            if let Err(e) = self.detect_and_attach_external() {
                log::warn!("Error during external Syncthing detection: {}", e);
            }
        }
        if let Some(proc) = &mut self.syncthing_process {
            proc.is_running()        } else {
            false
        }
    }
    
    /// Starts the Syncthing process if it's not already running.
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or containing an error if the process couldn't be started
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
    }    /// Stops the Syncthing process if it's running.
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or containing an error if the process couldn't be stopped
    pub fn stop_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(proc) = &mut self.syncthing_process {
            proc.stop().map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to stop Syncthing: {}", e)))?;
            log::info!("Syncthing process stopped successfully.");
        }
        self.syncthing_process = None;
        Ok(())
    }
}
