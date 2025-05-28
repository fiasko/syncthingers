use std::fs;
use std::io;
use std::path::PathBuf;

/// Struct to manage application directories and paths in a stateful way.
#[derive(Clone)]
pub struct AppDirs {
    base_dir: PathBuf,
}

impl AppDirs {
    /// Create a new AppDirs instance, using the provided override or the default app data dir.
    pub fn new(override_dir: Option<PathBuf>) -> io::Result<Self> {
        let base_dir = match override_dir {
            Some(dir) => dir,
            None => {
                let mut data_dir = dirs::data_local_dir().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Could not determine app data directory",
                    )
                })?;
                data_dir.push("Syncthingers");
                data_dir
            }
        };
        Ok(Self { base_dir })
    }

    /// Ensure the base directory exists.
    pub fn ensure_exists(&self) -> io::Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)?;
            log::info!(
                "Created application directory at: {}",
                self.base_dir.display()
            );
        }
        Ok(())
    }

    /// Get the path to the configuration file.
    pub fn config_file_path(&self) -> PathBuf {
        self.base_dir.join("configuration.json")
    }

    /// Get the path to the log file.
    pub fn log_file_path(&self) -> PathBuf {
        self.base_dir.join("syncthingers.log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_app_dirs_ensure_exists() {
        let temp_dir = env::temp_dir().join("test_app_dirs_ensure");
        let app_dirs = AppDirs::new(Some(temp_dir.clone())).unwrap();
        app_dirs.ensure_exists().unwrap();
        assert!(temp_dir.exists());
    }

    #[test]
    fn test_config_file_path() {
        let temp_dir = env::temp_dir().join("test_app_dirs_config");
        let app_dirs = AppDirs::new(Some(temp_dir.clone())).unwrap();
        let config_path = app_dirs.config_file_path();
        assert_eq!(config_path, temp_dir.join("configuration.json"));
    }

    #[test]
    fn test_log_file_path() {
        let temp_dir = env::temp_dir().join("test_app_dirs_log");
        let app_dirs = AppDirs::new(Some(temp_dir.clone())).unwrap();
        let log_path = app_dirs.log_file_path();
        assert_eq!(log_path, temp_dir.join("syncthingers.log"));
    }
}
