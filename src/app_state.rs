use crate::config::Config;
use crate::process::SyncthingProcess;

/// Syncthingers application state.
pub struct AppState {
    pub config: Config,
    pub syncthing_process: Option<SyncthingProcess>,
}

impl AppState {
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
    
    /// Starts the Syncthing process if it's not already running.
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
    
    /// Stops the Syncthing process if it's running.
    pub fn stop_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(proc) = &mut self.syncthing_process {
            proc.stop().map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to stop Syncthing: {}", e)))?;
            log::info!("Syncthing process stopped successfully.");
        }
        self.syncthing_process = None;
        Ok(())
    }

    /// Handles process closure on application exit based on configuration.
    /// 
    /// This method implements the configured process closure behavior:
    /// - CloseAll: Stops both managed and external Syncthing processes
    /// - CloseManaged: Only stops processes started by this app
    /// - DontClose: Leaves all processes running
    pub fn handle_exit_closure(&mut self) -> Result<(), crate::error_handling::AppError> {
        match self.config.process_closure_behavior {
            crate::config::ProcessClosureBehavior::CloseAll => {
                log::info!("Exit closure behavior: Closing all Syncthing processes");
                self.stop_all_syncthing_processes()
            },
            crate::config::ProcessClosureBehavior::CloseManaged => {
                log::info!("Exit closure behavior: Closing only managed processes");
                self.stop_managed_syncthing_processes()
            },
            crate::config::ProcessClosureBehavior::DontClose => {
                log::info!("Exit closure behavior: Leaving all processes running");
                Ok(())
            },
        }
    }

    /// Stops all Syncthing processes (both managed and external).
    fn stop_all_syncthing_processes(&mut self) -> Result<(), crate::error_handling::AppError> {
        // First stop our managed process if any
        let _ = self.stop_syncthing();

        // Then try to stop any external processes
        if let Err(e) = self.stop_external_syncthing_processes() {
            log::warn!("Failed to stop external Syncthing processes: {}", e);
            // Don't return error here as we want to try our best effort
        }

        Ok(())
    }

    /// Stops only processes that were started by this application.
    fn stop_managed_syncthing_processes(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(proc) = &self.syncthing_process {
            if proc.started_by_app {
                self.stop_syncthing()
            } else {
                log::info!("Leaving external Syncthing process running");
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// Attempts to stop external Syncthing processes.
    fn stop_external_syncthing_processes(&self) -> Result<(), crate::error_handling::AppError> {
        // Try to find and stop any external processes
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true) {
            Ok(Some(mut proc)) => {
                proc.stop().map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to stop external Syncthing: {}", e)))?;
                log::info!("Stopped external Syncthing process");
                Ok(())
            },
            Ok(None) => {
                log::debug!("No external Syncthing process found to stop");
                Ok(())
            },
            Err(e) => {
                Err(crate::error_handling::AppError::ProcessError(format!("Failed to detect external Syncthing: {}", e)))
            }        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProcessClosureBehavior;

    fn create_test_config(closure_behavior: ProcessClosureBehavior) -> Config {
        Config {
            log_level: "info".to_string(),
            syncthing_path: "test_syncthing.exe".to_string(),
            web_ui_url: "http://localhost:8384".to_string(),
            startup_args: vec![],
            process_closure_behavior: closure_behavior,
        }
    }

    #[test]
    fn test_handle_exit_closure_dont_close() {
        let config = create_test_config(ProcessClosureBehavior::DontClose);
        let mut app_state = AppState::new(config);
        
        // Should succeed without doing anything
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_no_process() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config);
        
        // Should succeed when no process is running
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_with_external() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config);
        
        // Simulate an external process
        app_state.syncthing_process = Some(crate::process::SyncthingProcess::mock_for_testing(false));
        
        // Should succeed and leave external process running
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_with_managed() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config);
        
        // Simulate a managed process
        app_state.syncthing_process = Some(crate::process::SyncthingProcess::mock_for_testing(true));
        
        // Should succeed and stop the managed process
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        assert!(app_state.syncthing_process.is_none());
    }

    #[test]
    fn test_handle_exit_closure_close_all() {
        let config = create_test_config(ProcessClosureBehavior::CloseAll);
        let mut app_state = AppState::new(config);
        
        // Simulate a process
        app_state.syncthing_process = Some(crate::process::SyncthingProcess::mock_for_testing(false));
        
        // Should succeed and stop all processes
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        assert!(app_state.syncthing_process.is_none());
    }
}
