use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// Module responsible for managing application directories and paths.
/// This provides a platform-independent API for accessing configuration
/// and log file locations.

/// Gets the base application directory for user-specific data.
/// On Windows, this is typically %LOCALAPPDATA%\Syncthingers
/// (e.g., C:\Users\username\AppData\Local\Syncthingers)
pub fn get_app_data_dir() -> Option<PathBuf> {
    let mut data_dir = dirs::data_local_dir()?;
    data_dir.push("Syncthingers");
    Some(data_dir)
}

/// Gets the configuration directory.
/// If override_path is provided, uses that instead of the default.
pub fn get_config_dir(override_path: Option<&Path>) -> Option<PathBuf> {
    match override_path {
        Some(path) => Some(path.to_path_buf()),
        None => get_app_data_dir()
    }
}

/// Gets the log directory.
/// If override_path is provided, uses that instead of the default.
pub fn get_log_dir(override_path: Option<&Path>) -> Option<PathBuf> {
    match override_path {
        Some(path) => Some(path.to_path_buf()),
        None => get_app_data_dir()
    }
}

/// Ensures all required application directories exist.
/// Creates them if they don't.
pub fn ensure_app_dirs_exist() -> io::Result<()> {
    if let Some(dir) = get_app_data_dir() {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
            log::info!("Created application directory at: {}", dir.display());
        }
    }
    
    Ok(())
}

/// Returns the path to the configuration file.
/// If override_path is provided, uses that path directly.
pub fn get_config_file_path(override_path: Option<&Path>) -> Option<PathBuf> {
    let config_dir = match override_path {
        Some(path) => path.to_path_buf(),
        None => {
            let dir = get_config_dir(None)?;
            // Create directory if it doesn't exist
            if !dir.exists() {
                let _ = fs::create_dir_all(&dir);
            }
            dir
        }
    };
    
    // If override_path is a file, return it directly
    if override_path.is_some() && override_path.unwrap().is_file() {
        return override_path.map(|p| p.to_path_buf());
    }
    
    Some(config_dir.join("configuration.json"))
}

/// Returns the path to the log file.
pub fn get_log_file_path(override_path: Option<&Path>) -> Option<PathBuf> {
    let log_dir = match override_path {
        Some(path) => {
            if path.is_file() {
                // If override_path is a file, return it directly
                return Some(path.to_path_buf());
            }
            path.to_path_buf()
        },
        None => {
            let dir = get_log_dir(None)?;
            // Create directory if it doesn't exist
            if !dir.exists() {
                let _ = fs::create_dir_all(&dir);
            }
            dir
        }
    };
    
    Some(log_dir.join("syncthingers.log"))
}

/// Migrates existing configuration and log files from the application
/// directory to the user data directory, if necessary.
pub fn migrate_from_exe_dir() -> io::Result<bool> {
    let app_dir = get_app_data_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine app data directory")
    })?;
    
    // Check if the target directory already has config files
    let target_config_path = app_dir.join("configuration.json");
    
    if target_config_path.exists() {
        // Already migrated
        log::debug!("Configuration already exists in user directory, no migration needed");
        return Ok(false);
    }
    
    // Try to get current executable directory
    let exe_dir = match std::env::current_exe() {
        Ok(exe_path) => match exe_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => return Ok(false)
        },
        Err(_) => return Ok(false)
    };
    
    // Check for configuration in executable directory
    let source_config_path = exe_dir.join("configuration.json");
    
    if source_config_path.exists() {
        // Ensure app directory exists
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)?;
        }
        
        // Copy configuration file
        fs::copy(&source_config_path, &target_config_path)?;
        log::info!("Migrated configuration from {} to {}", 
            source_config_path.display(), target_config_path.display());
            
        // Also copy log file if it exists
        let source_log_path = exe_dir.join("syncthingers.log");
        if source_log_path.exists() {
            let target_log_path = app_dir.join("syncthingers.log");
            fs::copy(&source_log_path, &target_log_path)?;
            log::info!("Migrated log file from {} to {}", 
                source_log_path.display(), target_log_path.display());
        }
        
        return Ok(true);
    }
    
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_get_app_data_dir() {
        let app_dir = get_app_data_dir();
        assert!(app_dir.is_some());
        let dir = app_dir.unwrap();
        assert!(dir.ends_with("Syncthingers"));
    }
    
    #[test]
    fn test_get_config_file_path_with_override() {
        let temp_path = env::temp_dir().join("test_config.json");
        let config_path = get_config_file_path(Some(&temp_path));
        assert!(config_path.is_some());
        assert_eq!(config_path.unwrap(), temp_path);
    }
    
    #[test]
    fn test_get_config_file_path_default() {
        let config_path = get_config_file_path(None);
        assert!(config_path.is_some());
        let path = config_path.unwrap();
        assert!(path.ends_with("configuration.json"));
        assert!(path.to_string_lossy().contains("Syncthingers"));
    }
}
