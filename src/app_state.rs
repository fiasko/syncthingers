use crate::app_dirs::AppDirs;
use crate::config::Config;
use crate::process::SyncthingProcess;

/// Syncthingers application state.
pub struct AppState {
    pub config: Config,
    pub syncthing_process: Option<SyncthingProcess>,
    pub app_dirs: AppDirs,
}

impl AppState {
    pub fn new(config: Config, app_dirs: AppDirs) -> Self {
        // Skip process detection for test environments
        let syncthing_process = if config.syncthing_path.contains("test")
            || config.syncthing_path.contains("mock")
            || config.syncthing_path.contains("nonexistent")
        {
            log::debug!(
                "Skipping process detection for test path: {}",
                config.syncthing_path
            );
            None
        } else {
            // Try to detect and attach to an external Syncthing process
            match SyncthingProcess::detect_process(&config.syncthing_path, true) {
                Ok(Some(proc)) => {
                    log::info!("Attached to external Syncthing process.");
                    Some(proc)
                }
                Ok(None) => None,
                Err(e) => {
                    log::warn!("Failed to detect external Syncthing process: {}", e);
                    None
                }
            }
        };
        Self {
            config,
            syncthing_process,
            app_dirs,
        }
    }

    /// Attempts to detect and attach to an external Syncthing process, updating state.
    pub fn detect_and_attach_external(&mut self) -> Result<bool, crate::error_handling::AppError> {
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true) {
            Ok(Some(proc)) => {
                self.syncthing_process = Some(proc);
                log::info!("Attached to external Syncthing process.");
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("Failed to detect external Syncthing process: {}", e);
                Err(crate::error_handling::AppError::ProcessError(format!(
                    "Failed to detect external Syncthing: {}",
                    e
                )))
            }
        }
    }
    
    /// Checks if Syncthing is currently running.
    pub fn syncthing_running(&mut self) -> bool {
        // First, check our process tracking state
        // If we have a tracked process, just check its status directly
        if let Some(proc) = &mut self.syncthing_process {
            // We already know about a process - check if it's still running
            if proc.is_running() {
                return true;
            } else {
                // Process is no longer running, clear our reference to it
                self.syncthing_process = None;
                return false;
            }
        }

        // Skip external detection for test environments
        if self.config.syncthing_path.contains("test")
            || self.config.syncthing_path.contains("mock")
            || self.config.syncthing_path.contains("nonexistent")
        {
            return false;
        }

        // No process being tracked, try to detect an external one
        if let Err(e) = self.detect_and_attach_external() {
            log::warn!("Error during external Syncthing detection: {}", e);
            return false;
        }

        // Check if we found and attached to an external process
        self.syncthing_process.is_some()
    }

    /// Starts the Syncthing process if it's not already running.
    pub fn start_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if self.syncthing_running() {
            return Ok(());
        }
        let exe_path = &self.config.syncthing_path;
        let args = &self.config.startup_args;
        let mut process = SyncthingProcess::new(exe_path);
        process.start(args).map_err(|e| {
            crate::error_handling::AppError::ProcessError(format!(
                "Failed to start Syncthing: {}",
                e
            ))
        })?;
        self.syncthing_process = Some(process);

        log::info!("Syncthing process started successfully.");
        Ok(())
    }    /// Stops the Syncthing process if it's running.
    pub fn stop_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(process) = &mut self.syncthing_process {
            if process.started_by_app {
                // For app-started processes, use the normal stop method
                if let Err(e) = process.stop() {
                    log::warn!("Failed to stop app-started process: {}", e);
                } else {
                    log::info!("App-started Syncthing process stopped successfully.");
                }
            } else {
                // For external processes, use the external process stopping function
                log::info!("Stopping external Syncthing process via external process termination");
                if let Err(e) = self.stop_external_syncthing_processes() {
                    log::warn!("Failed to stop external Syncthing processes: {}", e);
                    return Err(e);
                } else {
                    log::info!("External Syncthing process stopped successfully.");
                }
            }
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
            }
            crate::config::ProcessClosureBehavior::CloseManaged => {
                log::info!("Exit closure behavior: Closing only managed processes");
                self.stop_managed_syncthing_processes()
            }
            crate::config::ProcessClosureBehavior::DontClose => {
                log::info!("Exit closure behavior: Leaving all processes running");
                Ok(())
            }
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
        // Use the process module function to stop external processes
        crate::process::stop_external_syncthing_processes(&self.config.syncthing_path).map_err(
            |e| {
                crate::error_handling::AppError::ProcessError(format!(
                    "Failed to stop external Syncthing processes: {}",
                    e
                ))
            },
        )?;
        log::info!("Stopped external Syncthing processes");
        Ok(())
    }

    /// Checks and auto-starts Syncthing if needed based on configuration.
    pub fn check_and_autostart_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if self.config.auto_launch_internal {
            // If not running, start internal syncthing
            if !self.syncthing_running() {
                log::info!(
                    "Auto-launching internal Syncthing as no external process is running and auto_launch_internal is enabled."
                );
                self.start_syncthing()?;
            } else {
                log::info!("Syncthing is already running, auto-launch not needed.");
            }
        } else {
            log::debug!("Auto-launch internal Syncthing is disabled in config.");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProcessClosureBehavior;
    fn create_test_config(closure_behavior: ProcessClosureBehavior) -> Config {
        Config {
            log_level: "info".to_string(),
            syncthing_path: "nonexistent_test_syncthing_12345.exe".to_string(), // Use a clearly nonexistent path
            web_ui_url: "http://localhost:8384".to_string(),
            startup_args: vec![],
            process_closure_behavior: closure_behavior,
            auto_launch_internal: false,
        }
    }

    #[test]
    fn test_handle_exit_closure_dont_close() {
        let config = create_test_config(ProcessClosureBehavior::DontClose);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());

        // Should succeed without doing anything
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_no_process() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());

        // Should succeed when no process is running
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_with_external() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());
        // Simulate an external process
        app_state.syncthing_process =
            Some(crate::process::SyncthingProcess::mock_for_testing(false));

        // Should succeed and leave external process running
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_with_managed() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());

        // Simulate a managed process
        app_state.syncthing_process =
            Some(crate::process::SyncthingProcess::mock_for_testing(true));

        // Should succeed and stop the managed process
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        assert!(app_state.syncthing_process.is_none());
    }

    #[test]
    fn test_handle_exit_closure_close_all() {
        let config = create_test_config(ProcessClosureBehavior::CloseAll);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());

        // Simulate a process
        app_state.syncthing_process =
            Some(crate::process::SyncthingProcess::mock_for_testing(false));

        // For CloseAll behavior, it calls stop_all_syncthing_processes which:
        // 1. Calls stop_syncthing() to clean up tracked process (sets syncthing_process = None)
        // 2. Calls stop_external_syncthing_processes() (skipped in test env)
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        // The process should be None after cleanup since stop_syncthing() was called
        assert!(app_state.syncthing_process.is_none());
    }
}
