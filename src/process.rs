use std::process::{Child, Command, Stdio};
use std::io;

pub struct SyncthingProcess {
    child: Option<Child>,
    pub started_by_app: bool,
}

impl SyncthingProcess {
    pub fn start(exe_path: &str, args: &[String]) -> io::Result<Self> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW = 0x08000000
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let child = Command::new(exe_path)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()?;
            return Ok(Self {
                child: Some(child),
                started_by_app: true,
            });
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
        if let Some(child) = &mut self.child {
            child.kill()?;
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
                return Ok(Some(Self { child: None, started_by_app: false }));
            } else {
                log::info!("No running Syncthing process detected for path: {}", exe_path);
            }
        }
        Ok(None)
    }
}
