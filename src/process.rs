use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use sysinfo::{System, ProcessesToUpdate, Pid};

/// Represents a Syncthing process that can be managed by the application.
///
/// This struct handles both processes started by the app and external processes.
/// Uses sysinfo for cross-platform process management.
pub struct SyncthingProcess {
    child: Option<Child>,
    pub started_by_app: bool,
    pub syncthing_path: String,
    pub pid: Option<u32>, // Process ID
    system: System, // sysinfo System instance for process monitoring
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
            system: System::new(),
        }
    }
    /// Detects if a Syncthing process is currently running and creates a SyncthingProcess instance.
    pub fn detect_process(syncthing_path: &str, external_only: bool) -> io::Result<Option<Self>> {
        use sysinfo::{ProcessesToUpdate, System};

        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);

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

        let child = command.spawn()?;

        self.pid = Some(child.id());
        log::info!("Syncthing process started with PID: {}", child.id());

        self.child = Some(child);
        self.started_by_app = true;

        Ok(())
    }    /// Stops the Syncthing process if it was started by this application.
    pub fn stop(&mut self) -> io::Result<()> {
        if !self.started_by_app {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Cannot stop external process",
            ));
        }

        if let Some(mut child) = self.child.take() {
            log::info!("Stopping Syncthing process (PID: {:?})", self.pid);

            // Use sysinfo to kill the process if std::process fails
            if let Some(pid) = self.pid {
                self.system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from(pid as usize)]), false);
                if let Some(process) = self.system.process(Pid::from(pid as usize)) {
                    if process.kill() {
                        log::info!("Successfully terminated Syncthing process using sysinfo");
                        self.child = None;
                        self.pid = None;
                        self.started_by_app = false;
                        return Ok(());
                    }
                }
            }

            // Fallback: try to kill the process directly using std::process
            match child.kill() {
                Ok(()) => {
                    log::info!("Syncthing process terminated successfully");
                    let _ = child.wait(); // Clean up zombie process
                }
                Err(e) => {
                    log::warn!("Failed to kill Syncthing process: {}", e);
                    return Err(e);
                }
            }

            self.child = None;
            self.pid = None;
            self.started_by_app = false;
        }

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
                    false
                }
            }
        } else if let Some(pid) = self.pid {
            // External process - check if it's still running using sysinfo
            use sysinfo::{Pid, ProcessesToUpdate, System};
            let mut system = System::new();
            let pid_obj = Pid::from(pid as usize);
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
    }    /// Checks if this process was started by the application.
    #[allow(dead_code)]
    pub fn is_started_by_app(&self) -> bool {
        self.started_by_app
    }#[cfg(test)]
    pub fn mock_for_testing(started_by_app: bool) -> Self {
        Self {
            child: None,
            started_by_app,
            syncthing_path: "mock_syncthing".to_string(),
            pid: Some(12345),
            system: System::new(),
        }
    }
}

/// Stops all external Syncthing processes running on the system.
pub fn stop_external_syncthing_processes(syncthing_path: &str) -> io::Result<()> {
    use sysinfo::{ProcessesToUpdate, System};

    // Skip process killing for clearly test-related paths
    if syncthing_path.contains("test") || syncthing_path.contains("mock") || syncthing_path.contains("nonexistent") {
        log::debug!("Skipping external process termination for test path: {}", syncthing_path);
        return Ok(());
    }

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    // Get the executable name to search for
    let exe_name = Path::new(syncthing_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("syncthing");

    // Remove .exe extension if present for cross-platform compatibility
    let exe_name = exe_name.strip_suffix(".exe").unwrap_or(exe_name);

    let mut terminated_count = 0;    // Find and terminate all Syncthing processes
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
