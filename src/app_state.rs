use crate::app_dirs::AppDirs;
use crate::config::{Config, ProcessClosureBehavior};
use crate::error_handling::AppError;
use crate::process;
use crate::process::SyncthingProcess;
use crate::utils::is_test_environment;

/// Syncthingers application state.
pub struct AppState {
    pub config: Config,
    pub syncthing_process: Option<SyncthingProcess>,
    pub app_dirs: AppDirs,
}

impl AppState {
    pub fn new(config: Config, app_dirs: AppDirs) -> Self {
        // Skip process detection for test environments
        let syncthing_process = if is_test_environment(&config.syncthing_path) {
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
    pub fn detect_and_attach_external(&mut self) -> Result<bool, AppError> {
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true) {
            Ok(Some(proc)) => {
                self.syncthing_process = Some(proc);
                log::info!("Attached to external Syncthing process.");
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("Failed to detect external Syncthing process: {}", e);
                Err(AppError::Process(format!(
                    "Failed to detect external Syncthing: {}",
                    e
                )))
            }
        }
    }

    /// Checks if Syncthing is currently running.
    /// This method only checks tracked processes and does not attempt to detect or attach to external instances.
    pub fn syncthing_running(&mut self) -> bool {
        // Check our tracked process state
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

        // No process being tracked
        false
    }

    /// Starts the Syncthing process if it's not already running.
    pub fn start_syncthing(&mut self) -> Result<(), AppError> {
        if self.syncthing_running() {
            return Ok(());
        }
        let exe_path = &self.config.syncthing_path;
        let args = &self.config.startup_args;
        let mut process = SyncthingProcess::new(exe_path);
        process
            .start(args)
            .map_err(|e| AppError::Process(format!("Failed to start Syncthing: {}", e)))?;
        self.syncthing_process = Some(process);

        log::info!("Syncthing process started successfully.");
        Ok(())
    }

    /// Stops the Syncthing process if it's running.
    pub fn stop_syncthing(&mut self) -> Result<(), AppError> {
        match &self.syncthing_process {
            Some(process) => {
                if process.started_by_app {
                    // For app-started processes, use the normal stop method
                    self.stop_managed_syncthing_processes()?;
                } else {
                    // For external processes, use the external process stopping function
                    self.stop_external_syncthing_processes()?;
                }
            }
            None => {
                log::info!("No Syncthing process is currently tracked.");
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
    pub fn handle_exit_closure(&mut self) -> Result<(), AppError> {
        match self.config.process_closure_behavior {
            ProcessClosureBehavior::CloseAll => {
                log::info!("Exit closure behavior: Closing all Syncthing processes");
                _ = self.stop_managed_syncthing_processes();
                _ = self.stop_external_syncthing_processes();
            }
            ProcessClosureBehavior::CloseManaged => {
                log::info!("Exit closure behavior: Closing only managed processes");
                _ = self.stop_managed_syncthing_processes();
            }
            ProcessClosureBehavior::DontClose => {
                log::info!("Exit closure behavior: Leaving all processes running");
            }
        }

        self.syncthing_process = None;
        Ok(())
    }

    /// Stops only processes that were started by this application.
    fn stop_managed_syncthing_processes(&mut self) -> Result<(), AppError> {
        match &mut self.syncthing_process {
            Some(process) => {
                if process.started_by_app {
                    process.stop().map_err(|e| AppError::Process(format!("Failed to stop Syncthing: {}", e)))?;
                    log::info!("App-started Syncthing process stopped successfully.");
                }
            }
            None => {
                log::info!("No Syncthing process is currently tracked.");
            }
        }
        Ok(())
    }

    /// Attempts to stop external Syncthing processes.
    fn stop_external_syncthing_processes(&self) -> Result<(), AppError> {
        // Use the process module function to stop external processes
        process::stop_external_syncthing_processes(&self.config.syncthing_path).map_err(|e| {
            AppError::Process(format!(
                "Failed to stop external Syncthing processes: {}",
                e
            ))
        })?;
        log::info!("Stopped external Syncthing processes");
        Ok(())
    }

    /// Checks and auto-starts Syncthing if needed based on configuration.
    pub fn check_and_autostart_syncthing(&mut self) -> Result<(), AppError> {
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
            process_closure_behavior: closure_behavior,
            ..Config::default()
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
        app_state.syncthing_process = Some(SyncthingProcess::mock_for_testing(false));

        // Should succeed and leave external process running
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_with_managed() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());

        // Simulate a managed process
        app_state.syncthing_process = Some(SyncthingProcess::mock_for_testing(true));

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
        app_state.syncthing_process = Some(SyncthingProcess::mock_for_testing(false));

        // For CloseAll behavior, it calls stop_all_syncthing_processes which:
        // 1. Calls stop_syncthing() to clean up tracked process (sets syncthing_process = None)
        // 2. Calls stop_external_syncthing_processes() (skipped in test env)
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        // The process should be None after cleanup since stop_syncthing() was called
        assert!(app_state.syncthing_process.is_none());
    }
}
