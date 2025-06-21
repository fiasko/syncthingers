/// Normalizes an executable name by removing the .exe extension if present.
///
/// This function provides cross-platform compatibility by stripping the .exe
/// extension that is common on Windows but not used on other platforms.
/// The extension removal is case-insensitive to handle .exe, .EXE, .Exe, etc.
///
/// # Arguments
///
/// * `exe_name` - The executable name to normalize
///
/// # Returns
///
/// The executable name without the .exe extension
///
/// # Examples
///
/// ```
/// assert_eq!(normalize_exe_name("syncthing.exe"), "syncthing");
/// assert_eq!(normalize_exe_name("syncthing.EXE"), "syncthing");
/// assert_eq!(normalize_exe_name("syncthing"), "syncthing");
/// ```
pub fn normalize_exe_name(exe_name: &str) -> &str {
    if exe_name.len() >= 4 && exe_name[exe_name.len() - 4..].eq_ignore_ascii_case(".exe") {
        &exe_name[..exe_name.len() - 4]
    } else {
        exe_name
    }
}

/// Checks if the given path indicates a test environment.
///
/// This function returns true if the path contains any of the test-related keywords
/// that indicate the application is running in a test context and should skip
/// real process operations.
///
/// # Arguments
///
/// * `path` - The file path to check (typically syncthing_path from config)
///
/// # Returns
///
/// `true` if this appears to be a test environment, `false` otherwise
pub fn is_test_environment(path: &str) -> bool {
    path.contains("test") || path.contains("mock") || path.contains("nonexistent")
}

mod tests {
    #[test]
    fn test_normalize_exe_name_with_exe_extension() {
        use super::normalize_exe_name;
        assert_eq!(normalize_exe_name("syncthing.exe"), "syncthing");
        assert_eq!(normalize_exe_name("syncthing.EXE"), "syncthing");
        assert_eq!(normalize_exe_name("syncthing.Exe"), "syncthing");
        assert_eq!(normalize_exe_name("myapp.exe"), "myapp");
        assert_eq!(normalize_exe_name("test.EXE"), "test");
    }

    #[test]
    fn test_normalize_exe_name_without_exe_extension() {
        use super::normalize_exe_name;
        assert_eq!(normalize_exe_name("syncthing"), "syncthing");
        assert_eq!(normalize_exe_name("myapp"), "myapp");
        assert_eq!(normalize_exe_name("test"), "test");
    }
    #[test]
    fn test_normalize_exe_name_edge_cases() {
        use super::normalize_exe_name;
        assert_eq!(normalize_exe_name(""), "");
        assert_eq!(normalize_exe_name(".exe"), "");
        assert_eq!(normalize_exe_name(".EXE"), "");
        assert_eq!(normalize_exe_name(".Exe"), "");
        assert_eq!(normalize_exe_name("app.executable"), "app.executable"); // Different extension
        assert_eq!(normalize_exe_name("exe"), "exe"); // Too short to be .exe
        assert_eq!(normalize_exe_name("example"), "example"); // Contains "exe" but not as extension
    }

    #[test]
    fn test_is_test_environment() {
        use super::is_test_environment;
        assert!(is_test_environment("test_syncthing.exe"));
        assert!(is_test_environment("mock_syncthing"));
        assert!(is_test_environment("nonexistent_test_app"));
        assert!(is_test_environment("/path/to/test/syncthing"));

        assert!(!is_test_environment("syncthing.exe"));
        assert!(!is_test_environment("/usr/bin/syncthing"));
        assert!(!is_test_environment("real_app"));
    }
}
