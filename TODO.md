# Syncthingers Singleton Tray App: Step-by-Step plan and general project instructions

# Global Best Practices
- The codebase will be designed with an optional platform-independent API in mind, so that future versions can support other operating systems, even though the first version will be Windows-only.
- Log key events and errors throughout the app.
- Ensure robust error handling, with user feedback via logs and native dialogs for critical errors.
- Centralize application state management (configuration, process handle, UI state).
- Use clear, maintainable code and document requirements/goals for each task.
- Test thoroughly for graceful shutdown, correct singleton behavior, and proper handling of configuration and process management.
- Follow Rust best practices for error handling and resource management.
- Ensure configuration system is forward-compatible with auto-updating mechanism for new fields.

# This checklist provides a step-by-step development guide for building the Syncthingers singleton system tray application for Windows in Rust.

## Project Initialization
- [x] Create a new Rust binary project
- [x] Add `#![windows_subsystem = "windows"]` to hide the console window

## Dependency Setup
- [x] Add system tray library (e.g., `tray-item`)
- [x] Add `serde` and `serde_json` for configuration
- [x] Add `opener` for launching URLs/files
- [x] Add `thiserror` for custom error handling
- [x] Add `log` and a logger implementation (e.g., `simplelog`)
- [x] Add platform-specific dependencies for singleton enforcement (`winapi` for Windows with features: `winuser synchapi errhandlingapi winnt handleapi minwindef winerror`, `fs2` for file lock on other platforms)
- [x] Add `sysinfo` for cross-platform process management and monitoring
- [x] Add `dirs` for platform-specific user directory access

## Singleton Enforcement (Portable)
- [x] Create a `singleton.rs` module with a portable `SingletonGuard` interface
- [x] Implement Windows singleton using a named Mutex (via `winapi`)
- [x] Implement a placeholder/file lock for other platforms (e.g., using `fs2`)
- [x] Use `SingletonGuard::acquire()` in `main.rs` for singleton enforcement
- [x] If singleton cannot be acquired, show user feedback (e.g., message box) and exit
- [x] If singleton is acquired, continue normal startup

## Configuration Management
- [x] Define `configuration.json` structure (logging level, Syncthing path, web UI URL, startup args)
- [x] Implement `Config` struct with serde serialization/deserialization
- [x] Load configuration from file or create default if missing
- [x] Make logging behavior configurable via config
- [x] Add feature to open config file from the app
- [x] Add command line argument to only create default config file if it doesn't exist and exit
- [x] When creating the default config file, try to detect `syncthing.exe` in the system PATH and use its path if found.
- [x] Add process closure behavior configuration option (close_all, close_managed, dont_close)
- [x] Implement automatic configuration updating when new fields are added to the Config struct
- [x] Add auto-launch internal Syncthing configuration option

## Real-time Configuration Monitoring
- [ ] Implement file system watcher for `configuration.json` changes
- [ ] Add automatic configuration reload when file is modified
- [ ] Implement configuration validation with detailed error reporting
- [ ] Add error popup display for invalid configuration files (shown only once per modification)
- [ ] Log detailed configuration error information to log file
- [ ] Apply configuration changes without requiring application restart
- [ ] Handle configuration file deletion and recreation scenarios

## Syncthing Process Management
- [x] Implement process management (start, stop, monitor Syncthing)
- [x] Store process handle for management
- [x] Handle errors (e.g., executable not found, failed to start)
- [x] Detect if Syncthing was started by this app
- [x] Monitor the Syncthing process: if `syncthing.exe` is killed or crashes, update the tray icon state accordingly.
 - [x] Implemented polling-based monitoring system with 2-second intervals using sysinfo
 - [x] Added cross-platform process detection and monitoring
- [x] Ensure that stopping Syncthing also terminates all child processes (migrated from Windows Job Objects to sysinfo for cross-platform process tree termination)
- [x] When starting `syncthing.exe`, ensure it does not open a terminal window (should be fully background/hidden).
- [x] Fix issue with command windows appearing when terminating external processes (CREATE_NO_WINDOW flag)
- [x] Enable stopping external Syncthing processes from tray menu
- [x] Implement comprehensive process tree tracking and termination using sysinfo
- [ ] Print Syncthing version to log file each time it's started (execute `syncthing --version` and log the output)

## System Tray UI
- [x] Add tray icon (running/stopped state)
    - [x] Use `assets/icons/syncthing_green.ico` for running state
    - [x] Use `assets/icons/syncthing_red.ico` for stopped state
- [x] Update icon and tooltip based on Syncthing status
- [x] Design tray menu:
    - [x] Status indicator
    - [x] Start/Stop Syncthing
    - [x] Open Syncthing Web UI
    - [x] Open App Configuration
    - [x] Exit
- [x] Implement menu actions and state updates
- [x] Implement tray menu action handling:
    - [x] Start/Stop Syncthing process from menu
    - [x] Open Syncthing Web UI in browser
    - [x] Open configuration file in editor
    - [x] Exit application cleanly with configurable process closure behavior

## Application State Management
- [x] Centralize app state (config, process handle, UI state)
- [x] Ensure thread-safe access (e.g., Arc<Mutex<_>>)
- [x] Implement cross-platform process management using sysinfo
- [x] Add comprehensive process tree tracking and termination

## Cross-Platform Process Management
- [x] Implement process tree discovery and tracking using sysinfo
- [x] Add cross-platform process termination with sysinfo's kill() method
- [x] Optimize process monitoring with minimal refresh flags for better performance
- [x] Add test environment detection to skip external process operations during testing
- [x] Implement external process detection and attachment capabilities
- [x] Add ability to stop external Syncthing processes from tray menu

## Error Handling & User Feedback
- [x] Use custom error types throughout
- [x] Log all errors
- [x] Show native dialog for critical errors (e.g., config missing, Syncthing not found)

## Windows-Specific Build & Packaging
- [x] Embed icons and version info in executable
- [x] Build release executable
- [x] Test by running the .exe directly
- [x] Document Syncthing path/config requirements

## Testing & Robustness
- [ ] Test singleton enforcement
- [ ] Test configuration loading and error cases
- [ ] Test process management (start/stop/restart Syncthing)
- [ ] Test tray UI and menu actions
- [ ] Test graceful shutdown and cleanup

## User Directory Configuration
- [x] Place settings files in user home directory on supported platforms
- [x] For Windows, store in AppData\Local\Syncthingers directory (using dirs crate)
- [x] Use this directory for log files and configuration in release builds
- [x] ~~Add `--config-path` argument to override the default location~~ (replaced with `--portable` flag)
- [x] Create directories if they don't exist on application startup
- [x] Add `--portable` flag to use current working directory for configuration and logs
- [x] Implement stateful AppDirs module for centralized directory management
- [x] Add auto-launch feature for internal Syncthing when external is not running

## Syncthing Transfer Speed Monitoring
- [ ] Add a configurable option in the configuration to enable/disable transfer speed monitoring
- [ ] Use Syncthing's REST API to fetch transfer speed data
- [ ] Display current transfer speeds in the tray menu or tooltip
- [ ] Log transfer speed data if enabled
- [ ] Make polling interval and display options configurable

## External Syncthing Management
- [ ] Add support for monitoring remote Syncthing instances
- [ ] Configure multiple Syncthing servers in configuration
- [ ] Provide status monitoring for all configured instances
- [ ] Enable basic remote control operations

## Future Enhancements (Optional)
- [x] Auto-start Syncthing with the app (implemented as auto_launch_internal config option)
- [ ] Real-time configuration file monitoring and hot-reload (partially planned - see Real-time Configuration Monitoring section)
- [ ] Start tray app with Windows
- [ ] Advanced Syncthing status detection
- [ ] Add a simple UI panel for logs/config
- [ ] Add a `--print-log` startup argument that makes the log printing also in terminal when running debug build
- [ ] Refactor app argument handling to use clap
- [ ] Figure out better way to track spawned syncthing child processes.

---
This checklist is based solely on the project plan and is intended to guide development from start to finish.
