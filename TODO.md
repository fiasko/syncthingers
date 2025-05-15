# Syncthingers Singleton Tray App: Step-by-Step Guide

This checklist provides a step-by-step development guide for building the Syncthingers singleton system tray application for Windows in Rust.

## Project Initialization
- [x] Create a new Rust binary project
- [x] Add `#![windows_subsystem = "windows"]` to hide the console window
- [x] Set up version control (e.g., git)

## Dependency Setup
- [x] Add system tray library (e.g., `tray-item`)
- [x] Add `serde` and `serde_json` for configuration
- [x] Add `opener` for launching URLs/files
- [x] Add `thiserror` for custom error handling
- [x] Add `log` and a logger implementation (e.g., `simplelog`)
- [x] Add platform-specific dependencies for singleton enforcement (`winapi` for Windows with features: `winuser synchapi errhandlingapi winnt handleapi minwindef winerror`, `fs2` for file lock on other platforms)

## Singleton Enforcement (Portable)
- [x] Create a `singleton.rs` module with a portable `SingletonGuard` interface
- [x] Implement Windows singleton using a named Mutex (via `winapi`)
- [x] Implement a placeholder/file lock for other platforms (e.g., using `fs2`)
- [x] Use `SingletonGuard::acquire()` in `main.rs` for singleton enforcement
- [x] If singleton cannot be acquired, show user feedback (e.g., message box) and exit
- [x] If singleton is acquired, continue normal startup

## Configuration Management
- [ ] Define `configuration.json` structure (logging, Syncthing path, web UI URL, startup args)
- [ ] Implement `Config` struct with serde serialization/deserialization
- [ ] Load configuration from file or create default if missing
- [ ] Make logging behavior configurable via config
- [ ] Add feature to open config file from the app

## Logging
- [ ] Initialize logging as per config (level, file path)
- [ ] Log key events and errors throughout the app

## Syncthing Process Management
- [ ] Implement process management (start, stop, monitor Syncthing)
- [ ] Store process handle for management
- [ ] Handle errors (e.g., executable not found, failed to start)
- [ ] Detect if Syncthing was started by this app

## System Tray UI
- [ ] Add tray icon (running/stopped state)
- [ ] Update icon and tooltip based on Syncthing status
- [ ] Design tray menu:
    - [ ] Status indicator
    - [ ] Start/Stop Syncthing
    - [ ] Open Syncthing Web UI
    - [ ] Open App Configuration
    - [ ] Exit
- [ ] Implement menu actions and state updates

## Application State Management
- [ ] Centralize app state (config, process handle, UI state)
- [ ] Ensure thread-safe access (e.g., Arc<Mutex<_>>)

## Error Handling & User Feedback
- [ ] Use custom error types throughout
- [ ] Log all errors
- [ ] Show native dialog for critical errors (e.g., config missing, Syncthing not found)

## Windows-Specific Build & Packaging
- [ ] Embed icons and version info in executable
- [ ] Build release executable
- [ ] Test by running the .exe directly
- [ ] Document Syncthing path/config requirements

## Testing & Robustness
- [ ] Test singleton enforcement
- [ ] Test configuration loading and error cases
- [ ] Test process management (start/stop/restart Syncthing)
- [ ] Test tray UI and menu actions
- [ ] Test graceful shutdown and cleanup

## Future Enhancements (Optional)
- [ ] Auto-start Syncthing with the app
- [ ] Start tray app with Windows
- [ ] Advanced Syncthing status detection
- [ ] Add a simple UI panel for logs/config

---
This checklist is based solely on the project plan and is intended to guide development from start to finish.
