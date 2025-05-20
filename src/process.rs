use std::process::{Child, Command, Stdio};
use std::io;

#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;
#[cfg(target_os = "windows")]
use winapi::um::jobapi2::{CreateJobObjectW, AssignProcessToJobObject};
#[cfg(target_os = "windows")]
use winapi::um::jobapi2::TerminateJobObject;
#[cfg(target_os = "windows")]
use winapi::um::handleapi::CloseHandle;

pub struct SyncthingProcess {
    child: Option<Child>,
    pub started_by_app: bool,
    #[cfg(target_os = "windows")]
    job_handle: Option<usize>, // Use usize for Send/Sync compatibility
}

impl SyncthingProcess {
    pub fn start(exe_path: &str, args: &[String]) -> io::Result<Self> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            use std::ptr;
            // CREATE_NO_WINDOW = 0x08000000
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let job = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null()) };
            if job.is_null() {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to create Job Object"));
            }
            let mut cmd = Command::new(exe_path);
            cmd.args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW);
            let child = cmd.spawn()?;
            let handle = child.as_raw_handle() as winapi::shared::ntdef::HANDLE;
            let ok = unsafe { AssignProcessToJobObject(job, handle) };
            if ok == 0 {
                unsafe { CloseHandle(job) };
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to assign process to Job Object"));
            }
            Ok(Self {
                child: Some(child),
                started_by_app: true,
                job_handle: Some(job as usize),
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let child = Command::new(exe_path)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            Ok(Self {
                child: Some(child),
                started_by_app: true,
            })
        }
    }

    /// Helper: Enumerate all running processes with the given name and their executable paths (Windows only)
    #[cfg(target_os = "windows")]
    fn enumerate_processes_by_name(process_name: &str) -> io::Result<Vec<(u32, String)>> {
        let output = Command::new("wmic")
            .args(["process", "where", &format!("name='{}'", process_name), "get", "ProcessId,ExecutablePath", "/format:csv"])
            .output()?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut result = Vec::new();
        for line in output_str.lines() {
            let parts: Vec<_> = line.split(',').collect();
            if parts.len() < 3 { continue; }
            let exe_path = parts[1].trim().to_string();
            let pid = parts[2].trim().parse::<u32>().ok();
            if let (Some(pid), exe_path) = (pid, exe_path) {
                result.push((pid, exe_path));
            }
        }
        Ok(result)
    }

    pub fn stop(&mut self) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            if let Some(job) = self.job_handle {
                unsafe {
                    TerminateJobObject(job as winapi::shared::ntdef::HANDLE, 1);
                    CloseHandle(job as winapi::shared::ntdef::HANDLE);
                }
                self.job_handle = None;
            } else if self.child.is_none() && !self.started_by_app {
                // If this is an external process, kill all syncthing.exe processes
                for (pid, _exe_path) in Self::enumerate_processes_by_name("syncthing.exe")? {
                    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
                }
            }
        }
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            self.child = None;
        }
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            child.try_wait().map(|o| o.is_none()).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn detect_existing(exe_path: &str) -> io::Result<Option<Self>> {
        #[cfg(target_os = "windows")]
        {
            use std::path::Path;
            let exe_filename = Path::new(exe_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            let exe_path_lower = exe_path.to_lowercase();
            let mut found_filename = false;
            for (_pid, path) in Self::enumerate_processes_by_name(&exe_filename)? {
                let path_lower = path.to_lowercase();
                if path_lower == exe_path_lower {
                    log::info!("Detected running Syncthing process with full path match: {}", path);
                    return Ok(Some(Self { child: None, started_by_app: false, job_handle: None }));
                }
                else if path_lower.ends_with(&exe_filename.to_lowercase()) {
                    found_filename = true;
                    log::info!("Found running process with matching filename: {} (full path: {})", exe_filename, path);
                }
            }
            if !found_filename {
                log::info!("No running process found with filename: {}", exe_filename);
            } else {
                log::info!("No running process found with full path match: {}", exe_path);
            }
        }
        Ok(None)
    }

    /// Detects an external Syncthing process (not started by this app) and returns a SyncthingProcess handle if found.
    pub fn detect_external(exe_path: &str) -> std::io::Result<Option<Self>> {
        // For now, just use detect_existing, but mark started_by_app as false and child as None
        #[cfg(target_os = "windows")]
        {
            if let Some(mut proc) = Self::detect_existing(exe_path)? {
                proc.started_by_app = false;
                proc.child = None; // We can't control the process
                #[cfg(target_os = "windows")] {
                    proc.job_handle = None;
                }
                return Ok(Some(proc));
            }
        }
        Ok(None)
    }
}
