use crate::config::Config;
use crate::process::SyncthingProcess;
use crate::process_monitor::ProcessEvent;
use crate::app_dirs::AppDirs;
use std::sync::mpsc::Sender;

/// Syncthingers application state.
pub struct AppState {
    pub config: Config,
    pub syncthing_process: Option<SyncthingProcess>,
    process_state_sender: Option<Sender<ProcessEvent>>,
    pub app_dirs: AppDirs,
}

impl AppState {
    pub fn new(config: Config, app_dirs: AppDirs) -> Self {
        // Try to detect and attach to an external Syncthing process
        let syncthing_process = match SyncthingProcess::detect_process(
            &config.syncthing_path, true, None) {
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
            process_state_sender: None,
            app_dirs,
        }
    }

    /// Attempts to detect and attach to an external Syncthing process, updating state.
    pub fn detect_and_attach_external(&mut self) -> Result<bool, crate::error_handling::AppError> {
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true, None) {
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
    
    /// Checks if Syncthing is currently running.
    pub fn syncthing_running(&mut self) -> bool {
        // First, check our process tracking state
        // If we have a tracked process, just check its status directly
        if let Some(proc) = &mut self.syncthing_process {
            // We already know about a process - check if it's still running
            if proc.is_running() {
                return true;
            } else {
                // Process is no longer running but we haven't received the exit event yet
                // Clear our reference to it
                self.syncthing_process = None;
                
                // Send an exit event if we were tracking it
                if let Some(sender) = &self.process_state_sender {
                    if let Err(e) = sender.send(crate::process_monitor::ProcessEvent::StateChanged) {
                        log::warn!("Failed to send process state change event: {}", e);
                    }
                }
                return false;
            }
        }
        
        // No process being tracked, try to detect an external one
        if let Err(e) = self.detect_and_attach_external() {
            log::warn!("Error during external Syncthing detection: {}", e);
            return false;
        }
          // Check if we found and attached to an external process
        if let Some(_detected_proc) = &mut self.syncthing_process {
            // Send a state change event for the newly detected process
            if let Some(sender) = &self.process_state_sender {
                if let Err(e) = sender.send(crate::process_monitor::ProcessEvent::StateChanged) {
                    log::warn!("Failed to send process state change event: {}", e);
                }
            }
            true
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
        let mut process = SyncthingProcess::new(
            exe_path,
            None, // Don't pass app_state reference to avoid circular dependencies
        );
        process.start(args)
            .map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to start Syncthing: {}", e)))?;
        self.syncthing_process = Some(process);
        
        // Send process started event through the channel
        if let Some(sender) = &self.process_state_sender {
            if let Err(e) = sender.send(crate::process_monitor::ProcessEvent::Started) {
                log::warn!("Failed to send process start event: {}", e);
            } else {
                log::debug!("Sent process start event");
            }
        }
        
        log::info!("Syncthing process started successfully.");
        Ok(())
    }
    
    /// Stops the Syncthing process if it's running.
    pub fn stop_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(process) = &mut self.syncthing_process {
            process.stop().map_err(|e| crate::error_handling::AppError::ProcessError(format!("Failed to stop Syncthing: {}", e)))?;
            log::info!("Syncthing process stopped successfully.");
            
            // Send process exit event when manually stopped
            if let Some(sender) = &self.process_state_sender {
                if let Err(e) = sender.send(crate::process_monitor::ProcessEvent::Exited) {
                    log::warn!("Failed to send process exit event: {}", e);
                } else {
                    log::debug!("Sent process exit event for stopped process");
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
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true, None) {
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
            } 
        }
    }
    
    /// Register a sender for process state events
    pub fn register_process_state_sender(&mut self, sender: Sender<ProcessEvent>) {
        self.process_state_sender = Some(sender);
    }

    /// Handles process exit events.
    pub fn handle_process_exit(&mut self, pid: u32) {
        log::info!("Handling process exit for PID {}", pid);
        
        // Check if this is our managed process
        if let Some(proc) = &self.syncthing_process {
            if let Some(proc_pid) = proc.pid {
                if proc_pid == pid {
                    log::info!("Syncthing process (PID: {}) has exited", pid);
                    // Clear the process from state
                    self.syncthing_process = None;
                    
                    // Send process exit event through the channel
                    if let Some(sender) = &self.process_state_sender {
                        if let Err(e) = sender.send(crate::process_monitor::ProcessEvent::Exited) {
                            log::warn!("Failed to send process exit event: {}", e);
                        } else {
                            log::debug!("Sent process exit event for PID {}", pid);
                        }
                    }
                    return;
                }
            }
        }
        
        // If we get here, it might be an external process that exited
        // Check if we're currently tracking an external process
        if let Some(proc) = &self.syncthing_process {
            if !proc.started_by_app {
                // This might be our external process that exited
                // Get the path from current config to avoid cloning issues
                let config_path = self.config.syncthing_path.clone();
                
                // Try to detect if it's still running
                match crate::process::SyncthingProcess::detect_process(
                    &config_path, true, None) // We don't need to pass app_state here
                {
                    Ok(None) => {
                        // Process is gone, update our state
                        log::info!("External Syncthing process is no longer running");
                        self.syncthing_process = None;
                        
                        // Send process exit event through the channel
                        if let Some(sender) = &self.process_state_sender {
                            if let Err(e) = sender.send(crate::process_monitor::ProcessEvent::Exited) {
                                log::warn!("Failed to send process exit event: {}", e);
                            } else {
                                log::debug!("Sent process exit event for external process");
                            }
                        }
                    },
                    Ok(Some(_)) => {
                        // Process is still running (maybe a different one with same path)
                        log::debug!("External Syncthing process is still running (but not the one that exited)");
                    },
                    Err(e) => {
                        log::warn!("Error checking external Syncthing process status: {}", e);
                    }
                }
            }
        }
    }

    /// Checks and auto-starts Syncthing if needed based on configuration.
    pub fn check_and_autostart_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if self.config.auto_launch_internal {
            // If not running, start internal syncthing
            if !self.syncthing_running() {
                log::info!("Auto-launching internal Syncthing as no external process is running and auto_launch_internal is enabled.");
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
            syncthing_path: "test_syncthing.exe".to_string(),
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
        app_state.syncthing_process = Some(crate::process::SyncthingProcess::mock_for_testing(false));
        
        // Should succeed and leave external process running
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_exit_closure_close_managed_with_managed() {
        let config = create_test_config(ProcessClosureBehavior::CloseManaged);
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());
        
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
        let mut app_state = AppState::new(config, AppDirs::new(None).unwrap());
        
        // Simulate a process
        app_state.syncthing_process = Some(crate::process::SyncthingProcess::mock_for_testing(false));
        
        // Should succeed and stop all processes
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        assert!(app_state.syncthing_process.is_none());
    }
}
