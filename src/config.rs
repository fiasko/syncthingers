use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Defines the behavior when closing the application regarding Syncthing processes.
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ProcessClosureBehavior {
    /// Close all Syncthing processes (both managed and external)
    #[serde(rename = "close_all")]
    CloseAll,
    /// Close only processes managed by this app
    #[serde(rename = "close_managed")]
    #[default]
    CloseManaged,
    /// Don't close any Syncthing processes on exit
    #[serde(rename = "dont_close")]
    DontClose,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub log_level: String,
    pub syncthing_path: String,
    pub web_ui_url: String,
    pub startup_args: Vec<String>,
    #[serde(default)]
    pub process_closure_behavior: ProcessClosureBehavior,
    pub auto_launch_internal: bool, // New field: auto-launch internal syncthing if external not running
}

impl Default for Config {
    fn default() -> Self {
        let syncthing_path = Self::find_syncthing_in_path()
            .unwrap_or_else(|| "C:/Program Files/Syncthing/syncthing.exe".to_string());
        Self {
            log_level: "info".to_string(),
            syncthing_path,
            web_ui_url: "http://localhost:8384".to_string(),
            startup_args: vec!["-no-browser".to_string()],
            process_closure_behavior: ProcessClosureBehavior::default(),
            auto_launch_internal: false, // Default: do not auto-launch
        }
    }
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

    /// Saves the current Config instance to the specified file path.
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let data = serde_json::to_string_pretty(self).unwrap();
        let mut file = fs::File::create(&path)?;
        file.write_all(data.as_bytes())?;
        log::info!("Config file saved at: {}", path.as_ref().display());
        Ok(())
    }

    pub fn load_or_create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if !path.as_ref().exists() {
            log::info!("Config file created at: {}", path.as_ref().display());
            let config = Self::default();
            config.save_to_file(&path)?;
            return Ok(config);
        }

        log::info!("Config file read from: {}", path.as_ref().display());

        let data = fs::read_to_string(&path)?;

        // Use serde_json::Value first to handle missing fields gracefully
        let json_value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Check if any fields are missing in the JSON
        if Self::check_missing_fields(&json_value) {
            log::debug!("Configuration is missing fields - updating with defaults");

            // Create default config
            let default_config = Self::default();

            // Create a merged config with default values for missing fields
            let merged = Self::merge_with_defaults(json_value, &default_config)?;

            // Write the updated config back to the file
            log::debug!("Updating config file with missing fields");
            merged.save_to_file(&path)?;
            Ok(merged)
        } else {
            // Try to deserialize into Config struct
            match serde_json::from_value(json_value) {
                Ok(cfg) => {
                    log::debug!("Configuration file loaded successfully");
                    Ok(cfg)
                }
                Err(err) => {
                    // If there's an error, it might be due to other issues
                    log::warn!("Error deserializing config: {}", err);
                    Err(io::Error::new(io::ErrorKind::InvalidData, err))
                }
            }
        }
    }

    /// Merges an existing config with default values, preserving existing settings
    /// while adding any missing fields from the default configuration.
    fn merge_with_defaults(existing: serde_json::Value, defaults: &Self) -> io::Result<Self> {
        // Convert defaults to Value for easier merging
        let default_value = serde_json::to_value(defaults)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Start with the defaults
        let mut merged = default_value;

        // Update with existing values where they exist
        if let Some(obj) = existing.as_object() {
            if let Some(merged_obj) = merged.as_object_mut() {
                for (key, value) in obj {
                    if !value.is_null() {
                        merged_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Convert the merged Value back to Config
        let config = serde_json::from_value(merged)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(config)
    }

    /// Checks if the JSON object is missing any fields that are present in the Config struct.
    /// Returns true if any fields are missing.
    fn check_missing_fields(json_value: &serde_json::Value) -> bool {
        // Check for specific fields we know might be missing
        // This is specifically checking for the process_closure_behavior and auto_launch_internal fields
        if let Some(obj) = json_value.as_object() {
            // If we're missing any expected field, return true
            if !obj.contains_key("process_closure_behavior") {
                log::info!("Missing field 'process_closure_behavior' in config");
                return true;
            }
            if !obj.contains_key("auto_launch_internal") {
                log::info!("Missing field 'auto_launch_internal' in config");
                return true;
            }
        }
        false
    }

    pub fn open_in_editor<P: AsRef<Path>>(path: P) -> io::Result<()> {
        opener::open(path.as_ref()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(
            config.process_closure_behavior,
            ProcessClosureBehavior::CloseManaged
        );
        assert!(!config.auto_launch_internal); // Test default value for the new field
    }

    #[test]
    fn test_merge_with_defaults() {
        // Create a partial config missing the new field
        let partial_json = r#"
        {
            "log_level": "debug",
            "syncthing_path": "test/path.exe",
            "web_ui_url": "http://localhost:9999",
            "startup_args": ["--test"]
        }
        "#;

        let partial_value: serde_json::Value = serde_json::from_str(partial_json).unwrap();
        let default_config = Config::default();

        // Merge the partial config with defaults
        let merged = Config::merge_with_defaults(partial_value, &default_config).unwrap();

        // Check that custom values were preserved
        assert_eq!(merged.log_level, "debug");
        assert_eq!(merged.syncthing_path, "test/path.exe");
        assert_eq!(merged.web_ui_url, "http://localhost:9999");
        assert_eq!(merged.startup_args, vec!["--test"]);

        // Check that missing fields were filled with defaults
        assert_eq!(
            merged.process_closure_behavior,
            ProcessClosureBehavior::CloseManaged
        );
        assert!(!merged.auto_launch_internal); // Test default value for the new field
    }

    #[test]
    fn test_load_with_missing_fields() -> io::Result<()> {
        // Create a temporary file with partial config
        let mut temp_file = NamedTempFile::new()?;
        let partial_json = r#"{
            "log_level": "warn",
            "syncthing_path": "test/syncthing.exe",
            "web_ui_url": "http://localhost:1234",
            "startup_args": []
        }"#;

        temp_file.write_all(partial_json.as_bytes())?;
        let path = temp_file.path().to_path_buf();

        // Load the config, which should add missing fields
        let config = Config::load_or_create(&path)?;

        // Verify the missing fields were added with default values
        assert_eq!(
            config.process_closure_behavior,
            ProcessClosureBehavior::CloseManaged
        );
        assert!(!config.auto_launch_internal); // Test default value for the new field

        // Original values should be preserved
        assert_eq!(config.log_level, "warn");
        assert_eq!(config.syncthing_path, "test/syncthing.exe");

        // Read back the file to verify it was updated
        let updated_data = fs::read_to_string(&path)?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_data)?;

        assert!(updated_json.get("process_closure_behavior").is_some());
        assert!(updated_json.get("auto_launch_internal").is_some());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_closure_behavior_default() {
        assert_eq!(
            ProcessClosureBehavior::default(),
            ProcessClosureBehavior::CloseManaged
        );
    }

    #[test]
    fn test_process_closure_behavior_serialization() {
        // Test serialization to JSON
        let behavior = ProcessClosureBehavior::CloseAll;
        let json = serde_json::to_string(&behavior).unwrap();
        assert_eq!(json, "\"close_all\"");

        let behavior = ProcessClosureBehavior::CloseManaged;
        let json = serde_json::to_string(&behavior).unwrap();
        assert_eq!(json, "\"close_managed\"");

        let behavior = ProcessClosureBehavior::DontClose;
        let json = serde_json::to_string(&behavior).unwrap();
        assert_eq!(json, "\"dont_close\"");
    }

    #[test]
    fn test_process_closure_behavior_deserialization() {
        // Test deserialization from JSON
        let behavior: ProcessClosureBehavior = serde_json::from_str("\"close_all\"").unwrap();
        assert_eq!(behavior, ProcessClosureBehavior::CloseAll);

        let behavior: ProcessClosureBehavior = serde_json::from_str("\"close_managed\"").unwrap();
        assert_eq!(behavior, ProcessClosureBehavior::CloseManaged);

        let behavior: ProcessClosureBehavior = serde_json::from_str("\"dont_close\"").unwrap();
        assert_eq!(behavior, ProcessClosureBehavior::DontClose);
    }

    #[test]
    fn test_config_with_process_closure_behavior() {
        let config = Config::default();
        assert_eq!(
            config.process_closure_behavior,
            ProcessClosureBehavior::CloseManaged
        );
        assert!(!config.auto_launch_internal); // Test default value for the new field
    }
}
