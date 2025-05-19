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

    pub fn stop(&mut self) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            if let Some(job) = self.job_handle {
                unsafe {
                    TerminateJobObject(job as winapi::shared::ntdef::HANDLE, 1);
                    CloseHandle(job as winapi::shared::ntdef::HANDLE);
                }
                self.job_handle = None;
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
        // On Windows, use tasklist to check for syncthing.exe
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let output = Command::new("tasklist").output()?;
            let output_str = String::from_utf8_lossy(&output.stdout);
            log::info!("Checking for existing Syncthing process with path: {}", exe_path);
            if output_str.to_lowercase().contains(&exe_path.to_lowercase()) {
                log::info!("Detected running Syncthing process: {}", exe_path);
                return Ok(Some(Self { child: None, started_by_app: false, job_handle: None }));
            } else {
                log::info!("No running Syncthing process detected for path: {}", exe_path);
            }
        }
        Ok(None)
    }
}
