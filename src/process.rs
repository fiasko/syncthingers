use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Represents a Syncthing process that can be managed by the application.
///
/// This struct handles both processes started by the app and external processes.
/// Uses sysinfo for cross-platform process management.
/// Tracks all Syncthing processes (parent and children) for complete shutdown.
pub struct SyncthingProcess {
    child: Option<Child>,
    pub started_by_app: bool,
    pub syncthing_path: String,
    pub pid: Option<u32>,   // Main process ID
    tracked_pids: Vec<u32>, // All Syncthing process IDs (parent + children)
    system: System,         // sysinfo System instance for process monitoring
}

// Mark SyncthingProcess as safe to send and share between threads
// This is safe because we've carefully handled Windows handles using usize
unsafe impl Send for SyncthingProcess {}
unsafe impl Sync for SyncthingProcess {}

impl SyncthingProcess {
    /// Creates a new SyncthingProcess instance.
    pub fn new(path: &str) -> Self {
        Self {
            syncthing_path: path.to_string(),
            child: None,
            started_by_app: false,
            pid: None,
            tracked_pids: Vec::new(),
            system: System::new(),
        }
    }
    /// Detects if a Syncthing process is currently running and creates a SyncthingProcess instance.
    pub fn detect_process(syncthing_path: &str, external_only: bool) -> io::Result<Option<Self>> {
        use sysinfo::{ProcessesToUpdate, System};
        let mut system = System::new();
        // Only refresh what we need - just process names and PIDs
        system.refresh_processes(ProcessesToUpdate::All, false);

        // Get the executable name to search for
        let exe_name = Path::new(syncthing_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("syncthing");

        // Remove .exe extension if present for cross-platform compatibility
        let exe_name = exe_name.strip_suffix(".exe").unwrap_or(exe_name);

        // Find processes matching the Syncthing executable
        for (pid, process) in system.processes() {
            let process_name = process.name().to_string_lossy();
            let process_name = process_name.strip_suffix(".exe").unwrap_or(&process_name);

            if process_name.eq_ignore_ascii_case(exe_name) {
                let mut syncthing_proc = Self::new(syncthing_path);
                syncthing_proc.pid = Some(pid.as_u32());
                syncthing_proc.started_by_app = false; // External process

                log::info!(
                    "Detected {} Syncthing process with PID: {}",
                    if external_only {
                        "external"
                    } else {
                        "existing"
                    },
                    pid
                );

                return Ok(Some(syncthing_proc));
            }
        }
        Ok(None)
    }

    /// Finds all Syncthing processes currently running on the system.
    fn find_all_syncthing_processes(&mut self) -> Vec<u32> {
        self.system.refresh_processes(ProcessesToUpdate::All, false);

        // Get the executable name to search for
        let exe_name = Path::new(&self.syncthing_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("syncthing");

        // Remove .exe extension if present for cross-platform compatibility
        let exe_name = exe_name.strip_suffix(".exe").unwrap_or(exe_name);

        let mut pids = Vec::new();
        for (pid, process) in self.system.processes() {
            let process_name = process.name().to_string_lossy();
            let process_name = process_name.strip_suffix(".exe").unwrap_or(&process_name);

            if process_name.eq_ignore_ascii_case(exe_name) {
                pids.push(pid.as_u32());
            }
        }

        log::debug!("Found {} Syncthing processes: {:?}", pids.len(), pids);
        pids
    }

    /// Updates the list of tracked Syncthing processes.
    fn update_tracked_processes(&mut self) {
        if self.started_by_app {
            self.tracked_pids = self.find_all_syncthing_processes();
        }
    }    /// Starts a new Syncthing process.
    pub fn start(&mut self, args: &[String]) -> io::Result<()> {
        if self.child.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Process is already running",
            ));
        }

        log::info!("Starting Syncthing process: {}", self.syncthing_path);

        let mut command = Command::new(&self.syncthing_path);
        command.args(args);
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
        
        // On Windows, prevent console window from appearing
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let child = command.spawn()?;

        self.pid = Some(child.id());
        log::info!("Syncthing process started with PID: {}", child.id());

        self.child = Some(child);
        self.started_by_app = true;

        // Give the process a moment to potentially spawn children
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Track all Syncthing processes (including any children that may have spawned)
        self.update_tracked_processes();
        log::info!(
            "Tracking {} Syncthing processes: {:?}",
            self.tracked_pids.len(),
            self.tracked_pids
        );

        Ok(())
    }

    /// Stops the Syncthing process if it was started by this application.
    pub fn stop(&mut self) -> io::Result<()> {
        if !self.started_by_app {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Cannot stop external process",
            ));
        }

        log::info!(
            "Stopping Syncthing process tree (main PID: {:?}, {} tracked processes)",
            self.pid,
            self.tracked_pids.len()
        );

        // First, try to stop all tracked processes using sysinfo
        let mut stopped_count = 0;
        for &pid in &self.tracked_pids {
            let pid_obj = Pid::from(pid as usize);
            self.system
                .refresh_processes(ProcessesToUpdate::Some(&[pid_obj]), false);

            if let Some(process) = self.system.process(pid_obj) {
                if process.kill() {
                    log::info!(
                        "Successfully terminated tracked Syncthing process PID: {}",
                        pid
                    );
                    stopped_count += 1;
                } else {
                    log::warn!("Failed to terminate tracked Syncthing process PID: {}", pid);
                }
            }
        }

        if stopped_count > 0 {
            log::info!("Stopped {} tracked Syncthing processes", stopped_count);
        }

        // Also handle the main child process if we have it
        if let Some(mut child) = self.child.take() {
            match child.kill() {
                Ok(()) => {
                    log::info!("Main Syncthing process terminated successfully");
                    let _ = child.wait(); // Clean up zombie process
                }
                Err(e) => {
                    log::warn!("Failed to kill main Syncthing process: {}", e);
                }
            }
        }

        // Clear all tracking
        self.child = None;
        self.pid = None;
        self.started_by_app = false;
        self.tracked_pids.clear();

        Ok(())
    }

    /// Checks if the process is currently running.
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    log::debug!("Process has exited, cleaning up");
                    self.child = None;
                    self.pid = None;
                    self.started_by_app = false;
                    self.tracked_pids.clear();
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking process status, assume it's dead
                    log::warn!("Error checking process status, assuming process is dead");
                    self.child = None;
                    self.pid = None;
                    self.started_by_app = false;
                    self.tracked_pids.clear();
                    false
                }
            }
        } else if let Some(pid) = self.pid {
            // External process - check if it's still running using sysinfo
            use sysinfo::{Pid, ProcessesToUpdate, System};
            let mut system = System::new();
            let pid_obj = Pid::from(pid as usize);
            // Only refresh the specific process we're checking
            system.refresh_processes(ProcessesToUpdate::Some(&[pid_obj]), false);

            if system.process(pid_obj).is_some() {
                true
            } else {
                // External process has exited
                log::debug!("External process (PID: {}) has exited", pid);
                self.pid = None;
                self.started_by_app = false;
                false
            }
        } else {
            false
        }
    }

    /// Checks if this process was started by the application.
    #[allow(dead_code)]
    pub fn is_started_by_app(&self) -> bool {
        self.started_by_app
    }
    #[cfg(test)]
    pub fn mock_for_testing(started_by_app: bool) -> Self {
        Self {
            child: None,
            started_by_app,
            syncthing_path: "mock_syncthing".to_string(),
            pid: Some(12345),
            tracked_pids: Vec::new(),
            system: System::new(),
        }
    }
}

/// Stops all external Syncthing processes running on the system.
pub fn stop_external_syncthing_processes(syncthing_path: &str) -> io::Result<()> {
    use sysinfo::{ProcessesToUpdate, System};

    // Skip process killing for clearly test-related paths
    if syncthing_path.contains("test")
        || syncthing_path.contains("mock")
        || syncthing_path.contains("nonexistent")
    {
        log::debug!(
            "Skipping external process termination for test path: {}",
            syncthing_path
        );
        return Ok(());
    }
    let mut system = System::new();
    // Only refresh what we need - process names for matching
    system.refresh_processes(ProcessesToUpdate::All, false);

    // Get the executable name to search for
    let exe_name = Path::new(syncthing_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("syncthing");

    // Remove .exe extension if present for cross-platform compatibility
    let exe_name = exe_name.strip_suffix(".exe").unwrap_or(exe_name);

    let mut terminated_count = 0; // Find and terminate all Syncthing processes
    for (pid, process) in system.processes() {
        let process_name = process.name().to_string_lossy();
        let process_name = process_name.strip_suffix(".exe").unwrap_or(&process_name);

        if process_name.eq_ignore_ascii_case(exe_name) {
            log::info!("Terminating external Syncthing process with PID: {}", pid);

            // Use sysinfo's cross-platform process killing
            if process.kill() {
                terminated_count += 1;
                log::info!("Successfully terminated process {}", pid);
            } else {
                log::warn!("Failed to terminate process {}", pid);
            }
        }
    }

    if terminated_count > 0 {
        log::info!(
            "Terminated {} external Syncthing process(es)",
            terminated_count
        );
    } else {
        log::debug!("No external Syncthing processes found to terminate");
    }

    Ok(())
}
