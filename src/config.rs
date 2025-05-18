use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::io::{self, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub log_level: String,
    pub syncthing_path: String,
    pub web_ui_url: String,
    pub startup_args: Vec<String>,
}

impl Config {
    pub fn find_syncthing_in_path() -> Option<String> {
        // On Windows, search for syncthing.exe in PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(';') {
                let exe_path = std::path::Path::new(dir).join("syncthing.exe");
                if exe_path.exists() {
                    log::info!("Found syncthing executable at: {}", exe_path.display());
                    return Some(exe_path.to_string_lossy().to_string());
                }
            }
        }
        None
    }

    pub fn default() -> Self {
        let syncthing_path = Self::find_syncthing_in_path()
            .unwrap_or_else(|| "C:/Program Files/Syncthing/syncthing.exe".to_string());
        Self {
            log_level: "info".to_string(),
            syncthing_path,
            web_ui_url: "http://localhost:8384".to_string(),
            startup_args: vec!["-no-browser".to_string()],
        }
    }

    pub fn load_or_create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if path.as_ref().exists() {
            log::info!("Config file read from: {}", path.as_ref().display());
            let data = fs::read_to_string(&path)?;
            let config: Self = serde_json::from_str(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(config)
        } else {
            log::info!("Config file created at: {}", path.as_ref().display());
            let config = Self::default();
            let data = serde_json::to_string_pretty(&config).unwrap();
            let mut file = fs::File::create(&path)?;
            file.write_all(data.as_bytes())?;
            Ok(config)
        }
    }

    pub fn open_in_editor<P: AsRef<Path>>(path: P) -> io::Result<()> {
        opener::open(path.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
