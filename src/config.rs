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
    pub fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            syncthing_path: "C:/Program Files/Syncthing/syncthing.exe".to_string(),
            web_ui_url: "http://localhost:8384".to_string(),
            startup_args: vec!["-no-browser".to_string()],
        }
    }

    pub fn load_or_create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if path.as_ref().exists() {
            let data = fs::read_to_string(&path)?;
            let config: Self = serde_json::from_str(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(config)
        } else {
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
