use crate::app_dirs::AppDirs;
use crate::config::Config;
use crate::process::SyncthingProcess;
use crate::process_monitor::ProcessEvent;
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
        let syncthing_process =
            SyncthingProcess::detect_process(&config.syncthing_path, true, None)
                .ok()
                .flatten();
        Self {
            config,
            syncthing_process,
            process_state_sender: None,
            app_dirs,
        }
    }

    pub fn detect_and_attach_external(&mut self) -> Result<bool, crate::error_handling::AppError> {
        match SyncthingProcess::detect_process(&self.config.syncthing_path, true, None) {
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

    pub fn syncthing_running(&mut self) -> bool {
        if let Some(proc) = &mut self.syncthing_process {
            if proc.is_running() {
                return true;
            } else {
                self.syncthing_process = None;
                if let Some(sender) = &self.process_state_sender {
                    let _ = sender.send(crate::process_monitor::ProcessEvent::StateChanged);
                }
                return false;
            }
        }
        if let Err(e) = self.detect_and_attach_external() {
            log::warn!("Error during external Syncthing detection: {}", e);
            return false;
        }
        if let Some(_detected_proc) = &mut self.syncthing_process {
            if let Some(sender) = &self.process_state_sender {
                let _ = sender.send(crate::process_monitor::ProcessEvent::StateChanged);
            }
            true
        } else {
            false
        }
    }

    pub fn start_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if self.syncthing_running() {
            return Ok(());
        }
        let mut process = SyncthingProcess::new(&self.config.syncthing_path, None);
        process.start(&self.config.startup_args).map_err(|e| {
            crate::error_handling::AppError::ProcessError(format!(
                "Failed to start Syncthing: {}",
                e
            ))
        })?;
        self.syncthing_process = Some(process);
        if let Some(sender) = &self.process_state_sender {
            let _ = sender.send(crate::process_monitor::ProcessEvent::Started);
        }
        log::info!("Syncthing process started successfully.");
        Ok(())
    }

    pub fn stop_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if let Some(process) = &mut self.syncthing_process {
            process.stop().map_err(|e| {
                crate::error_handling::AppError::ProcessError(format!(
                    "Failed to stop Syncthing: {}",
                    e
                ))
            })?;
            log::info!("Syncthing process stopped successfully.");
            if let Some(sender) = &self.process_state_sender {
                let _ = sender.send(crate::process_monitor::ProcessEvent::Exited);
            }
        }
        self.syncthing_process = None;
        Ok(())
    }

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

    fn stop_all_syncthing_processes(&mut self) -> Result<(), crate::error_handling::AppError> {
        let _ = self.stop_syncthing();
        if let Err(e) = SyncthingProcess::detect_process(&self.config.syncthing_path, true, None)
            .and_then(|opt| {
                if let Some(mut proc) = opt {
                    proc.stop()?;
                }
                Ok(())
            })
        {
            log::warn!("Failed to stop external Syncthing processes: {}", e);
        }
        Ok(())
    }

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

    pub fn register_process_state_sender(&mut self, sender: Sender<ProcessEvent>) {
        self.process_state_sender = Some(sender);
    }

    pub fn handle_process_exit(&mut self, pid: u32) {
        log::info!("Handling process exit for PID {}", pid);
        if let Some(proc) = &self.syncthing_process {
            if let Some(proc_pid) = proc.pid {
                if proc_pid == pid {
                    log::info!("Syncthing process (PID: {}) has exited", pid);
                    self.syncthing_process = None;
                    if let Some(sender) = &self.process_state_sender {
                        let _ = sender.send(crate::process_monitor::ProcessEvent::Exited);
                    }
                    return;
                }
            }
        }
        if let Some(proc) = &self.syncthing_process {
            if !proc.started_by_app {
                let config_path = self.config.syncthing_path.clone();
                match SyncthingProcess::detect_process(&config_path, true, None) {
                    Ok(None) => {
                        log::info!("External Syncthing process is no longer running");
                        self.syncthing_process = None;
                        if let Some(sender) = &self.process_state_sender {
                            let _ = sender.send(crate::process_monitor::ProcessEvent::Exited);
                        }
                    }
                    Ok(Some(_)) => {
                        log::debug!(
                            "External Syncthing process is still running (but not the one that exited)"
                        );
                    }
                    Err(e) => {
                        log::warn!("Error checking external Syncthing process status: {}", e);
                    }
                }
            }
        }
    }

    pub fn check_and_autostart_syncthing(&mut self) -> Result<(), crate::error_handling::AppError> {
        if self.config.auto_launch_internal {
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

        // Should succeed and stop all processes
        let result = app_state.handle_exit_closure();
        assert!(result.is_ok());
        assert!(app_state.syncthing_process.is_none());
    }
}
