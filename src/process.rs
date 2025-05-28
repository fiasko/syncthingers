use crate::app_state::AppState;
use crate::process_monitor::{ProcessMonitorHandle, register_process_exit_monitor};
use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, Weak};

#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use winapi::shared::ntdef::HANDLE;
#[cfg(target_os = "windows")]
use winapi::um::handleapi::CloseHandle;
#[cfg(target_os = "windows")]
use winapi::um::jobapi2::{AssignProcessToJobObject, CreateJobObjectW, TerminateJobObject};

/// Represents a Syncthing process that can be managed by the application.
///
/// This struct handles both processes started by the app and external processes.
pub struct SyncthingProcess {
    child: Option<Child>,
    pub started_by_app: bool,
    pub syncthing_path: String,
    pub pid: Option<u32>, // Process ID
    #[cfg(target_os = "windows")]
    job_handle: Option<usize>, // Use usize for Send/Sync compatibility
    monitor_handle: Option<ProcessMonitorHandle>, // New field for process monitoring
    app_state: Option<Weak<Mutex<AppState>>>, // Weak reference to AppState for callbacks
}

// Mark SyncthingProcess as safe to send and share between threads
// This is safe because we've carefully handled Windows handles using usize
unsafe impl Send for SyncthingProcess {}
unsafe impl Sync for SyncthingProcess {}

impl SyncthingProcess {
    /// Creates a new SyncthingProcess instance.
    pub fn new(path: &str, app_state: Option<Weak<Mutex<AppState>>>) -> Self {
        Self {
            syncthing_path: path.to_string(),
            child: None,
            started_by_app: false,
            pid: None,
            #[cfg(target_os = "windows")]
            job_handle: None,
            monitor_handle: None,
            app_state,
        }
    }

    /// Starts a new Syncthing process.
    pub fn start(&mut self, args: &[String]) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            use std::ptr;

            const CREATE_NO_WINDOW: u32 = 0x08000000;

            // Create a job object to manage the process
            let job = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null()) };
            if job.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to create Job Object",
                ));
            }

            // Start Syncthing process
            let child = Command::new(&self.syncthing_path)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()?;

            // Assign process to job for management
            let handle = child.as_raw_handle() as HANDLE;
            let ok = unsafe { AssignProcessToJobObject(job, handle) };

            if ok == 0 {
                unsafe { CloseHandle(job) };
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to assign process to Job Object",
                ));
            }
            let pid = child.id();
            self.child = Some(child);
            self.started_by_app = true;
            self.pid = Some(pid);
            self.job_handle = Some(job as usize);

            // Store the PID
            if let Some(child) = &self.child {
                if let Some(app_state_weak) = &self.app_state {
                    // Register the process for exit monitoring
                    let process_handle = child.as_raw_handle() as HANDLE;

                    match register_process_exit_monitor(
                        process_handle,
                        app_state_weak.clone(),
                        child.id(),
                    ) {
                        Ok(monitor) => {
                            self.monitor_handle = Some(monitor);
                            log::info!(
                                "Successfully registered exit monitor for Syncthing process (PID: {})",
                                child.id()
                            );
                        }
                        Err(e) => {
                            log::warn!("Failed to register exit monitor for Syncthing: {}", e);
                            // Continue without monitoring - we'll fallback to polling
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let child = Command::new(&self.syncthing_path)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;

            self.child = Some(child);
            self.started_by_app = true;
        }

        Ok(())
    }

    /// Enumerates all running processes with the given name and returns their PIDs and executable paths.
    #[cfg(target_os = "windows")]
    fn enumerate_processes_by_name(process_name: &str) -> io::Result<Vec<(u32, String)>> {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Run WMIC command to get process information
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("name='{}'", process_name),
                "get",
                "ProcessId,ExecutablePath",
                "/format:csv",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        // Parse the CSV output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut result = Vec::new();

        for line in output_str.lines() {
            let parts: Vec<_> = line.split(',').collect();

            // WMIC CSV format: Node,ExecutablePath,ProcessId
            if parts.len() < 3 {
                continue;
            }

            let exe_path = parts[1].trim().to_string();
            if let Ok(pid) = parts[2].trim().parse::<u32>() {
                result.push((pid, exe_path));
            }
        }

        Ok(result)
    }

    /// Stops the Syncthing process.
    pub fn stop(&mut self) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            // Define CREATE_NO_WINDOW constant for this method
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            if let Some(job) = self.job_handle {
                // For processes we created with a job object
                unsafe {
                    TerminateJobObject(job as HANDLE, 1);
                    CloseHandle(job as HANDLE);
                }
                self.job_handle = None;
            } else if self.child.is_none() && !self.started_by_app {
                // For external processes we're monitoring
                let filename = Path::new(&self.syncthing_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "syncthing.exe".to_string());

                log::debug!(
                    "Stopping external Syncthing process with filename: {}",
                    filename
                );

                // Find and kill the specific process matching our path
                for (pid, exe_path) in Self::enumerate_processes_by_name(&filename)? {
                    if exe_path.to_lowercase() == self.syncthing_path.to_lowercase() {
                        log::info!("Terminating external Syncthing process (PID: {})", pid);
                        let _ = Command::new("taskkill")
                            .args(["/PID", &pid.to_string(), "/F"])
                            .creation_flags(CREATE_NO_WINDOW)
                            .output();
                    }
                }
            }
        }

        // Standard child process termination
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            self.child = None;
        }
        Ok(())
    }

    /// Checks if the Syncthing process is currently running.
    pub fn is_running(&mut self) -> bool {
        match (&mut self.child, self.started_by_app) {
            // Child process started by us
            (Some(child), true) => {
                match child.try_wait() {
                    Ok(None) => true,     // Process still running
                    Ok(Some(_)) => false, // Process exited
                    Err(_) => false,      // Error checking process status
                }
            }

            // External process
            (None, false) => {
                // Check if the process with the same path still exists
                // Use the external_only=false since we only care if the process exists
                Self::detect_process(&self.syncthing_path, false, self.app_state.clone())
                    .map(|opt| opt.is_some())
                    .unwrap_or(false)
            }

            // Other cases (no child and started_by_app=true shouldn't happen)
            _ => false,
        }
    }

    /// Detects a Syncthing process with the given executable path.
    pub fn detect_process(
        exe_path: &str,
        external_only: bool,
        app_state: Option<Weak<Mutex<AppState>>>,
    ) -> io::Result<Option<Self>> {
        #[cfg(target_os = "windows")]
        {
            // Get just the filename from the path
            let exe_filename = Path::new(exe_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();

            if exe_filename.is_empty() {
                log::warn!("Invalid executable path: {}", exe_path);
                return Ok(None);
            }

            log::debug!("Detecting process with filename: {}", exe_filename);

            // Case-insensitive path comparison
            let exe_path_lower = exe_path.to_lowercase();

            // Check each running process with matching name
            for (pid, path) in Self::enumerate_processes_by_name(&exe_filename)? {
                let path_lower = path.to_lowercase();

                if path_lower == exe_path_lower {
                    let process_type = if external_only {
                        "external"
                    } else {
                        "existing"
                    };
                    log::info!(
                        "Found {} Syncthing process (PID: {}) at path: {}",
                        process_type,
                        pid,
                        path
                    );
                    let mut process = SyncthingProcess {
                        syncthing_path: exe_path.to_string(),
                        child: None,
                        started_by_app: false,
                        pid: Some(pid),
                        #[cfg(target_os = "windows")]
                        job_handle: None,
                        monitor_handle: None,
                        app_state,
                    };

                    #[cfg(target_os = "windows")]
                    {
                        // Try to open the process handle for monitoring
                        if let Some(app_state_weak) = &process.app_state {
                            use winapi::um::processthreadsapi::OpenProcess;
                            use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

                            unsafe {
                                let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
                                if !process_handle.is_null() {
                                    match register_process_exit_monitor(
                                        process_handle,
                                        app_state_weak.clone(),
                                        pid,
                                    ) {
                                        Ok(monitor) => {
                                            process.monitor_handle = Some(monitor);
                                            log::info!(
                                                "Successfully registered exit monitor for external Syncthing (PID: {})",
                                                pid
                                            );
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "Failed to register exit monitor for external Syncthing: {}",
                                                e
                                            );
                                            // Continue without monitoring - we'll fallback to polling
                                        }
                                    }

                                    // Close the process handle as it's been duplicated by the wait registration
                                    CloseHandle(process_handle);
                                } else {
                                    log::warn!(
                                        "Could not open handle to process {} for monitoring",
                                        pid
                                    );
                                }
                            }
                        }
                    }

                    return Ok(Some(process));
                }
            }

            log::debug!("No Syncthing process found at path: {}", exe_path);
        }

        Ok(None)
    }
    #[cfg(test)]
    /// Returns a mock Syncthing process for testing.
    ///
    /// This method is only available in test builds.
    pub fn mock_for_testing(started_by_app: bool) -> Self {
        Self {
            child: None,
            started_by_app,
            syncthing_path: "mock_syncthing.exe".to_string(),
            pid: Some(9999), // Mock PID for testing
            #[cfg(target_os = "windows")]
            job_handle: None,
            monitor_handle: None,
            app_state: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that a mock process can be created correctly
    #[test]
    fn test_mock_creation() {
        let proc = SyncthingProcess::mock_for_testing(true);
        assert!(proc.started_by_app);
        assert_eq!(proc.syncthing_path, "mock_syncthing.exe");
        assert!(proc.child.is_none());

        let external_proc = SyncthingProcess::mock_for_testing(false);
        assert!(!external_proc.started_by_app);
    }

    /// Tests that the is_running method returns false for mock processes
    #[test]
    fn test_mock_not_running() {
        let mut proc = SyncthingProcess::mock_for_testing(true);
        assert!(!proc.is_running());

        let mut external_proc = SyncthingProcess::mock_for_testing(false);
        assert!(!external_proc.is_running());
    }

    /// This simulates command execution by mocking parts of the system
    #[test]
    #[cfg(target_os = "windows")]
    fn test_process_detection_mock() {
        // This would need more sophisticated test infrastructure
        // with a mock for Command::new to properly test without
        // actual process execution

        // For a real implementation, you could use:
        // - mockall to mock Command
        // - a test-specific trait for executing commands
        // - creating a thin abstraction layer for system calls
    }
}
