# TODO List: Syncthingers

This document outlines the development plan for a Rust-based Windows system tray application to manage Syncthing.

## Phase 1: Project Setup & Core Dependencies

- [x] Initialize a new Rust project: `cargo new syncthingers`
- [x] Add `#![windows_subsystem = "windows"]` to `main.rs` to hide the console window on launch.
- [x] Research and choose `tray-item` system tray library.
- [x] Add `tray-item` to `Cargo.toml`.
- [x] Add `serde` and `serde_json` for configuration file handling: `cargo add serde serde_json --features serde/derive`
- [x] Add `opener` crate for opening web pages and files: `cargo add opener`
- [x] Add `thiserror` for custom error types: `cargo add thiserror`
- [x] Implement singleton instance check using a named mutex (Windows) in a modular way (`single_instance.rs`).
- [x] Add `log` and `simplelog` logger implementation: `cargo add log simplelog`.
- [x] Set up basic logging using `simplelog` (default file logger, conditional terminal logger).

## Phase 2: Configuration Management (`configuration.json`)

- [ ] Define the structure for `configuration.json`:
    - [ ] `syncthing_executable_path`: Full path to `syncthing.exe`.
    - [ ] `syncthing_gui_url`: URL for Syncthing's web UI (e.g., `http://127.0.0.1:8384`).
    - [ ] `syncthing_startup_args`: (Optional) Array of strings for arguments to pass to `syncthing.exe` on start.
    - [ ] `log_level`: (Optional) String for log level (e.g., "Info", "Debug", "Error", defaults to "Info").
    - [ ] `log_file_path`: (Optional) Path for the log file (e.g., defaults to a path in `%APPDATA%/<AppName>/app.log` or alongside the executable).
    - [ ] `config_file_path`: Path to this `configuration.json` file (for "Open Configuration" feature, could be self-determined).
- [ ] Make logging configurable (level, file path) via `configuration.json`, updating `logger::init_logging`.
- [ ] Implement a `Config` struct with `serde::Deserialize` and `serde::Serialize`.
- [ ] Implement a function to load `configuration.json`:
    - [ ] Determine path (e.g., alongside the executable, or in `%APPDATA%/<AppName>`).
    - [ ] If not found, create a default `configuration.json` with placeholder values.
    - [ ] Read and parse the file.
    - [ ] Handle I/O and parsing errors gracefully.
- [ ] Implement a function to save/create a default `configuration.json`.
- [ ] Ensure the application can locate its own configuration file for the "Open Configuration" feature.

## Phase 3: Syncthing Process Management

- [ ] Research and choose a method/crate for reliably finding external `syncthing.exe` processes by name or path (e.g., `sysinfo` crate).
- [ ] Create a module/struct for managing the Syncthing process (e.g., `SyncthingManager`).
- [ ] Implement `start_syncthing()`:
    - [ ] Read `syncthing_executable_path` and `syncthing_startup_args` from config.
    - [ ] Use `std::process::Command` to launch `syncthing.exe`.
    - [ ] Ensure it runs in the background (e.g., detached, no new console window if Syncthing itself doesn't handle this).
    - [ ] Store the `std::process::Child` handle to monitor and stop the process.
    - [ ] Handle errors (e.g., executable not found, failed to start).
- [ ] Implement `stop_syncthing()`:
    - [ ] Use the stored `Child` handle to kill the process.
    - [ ] If Syncthing was detected running externally, attempt to find its PID and terminate it.
    - [ ] On Windows, ensure it's a clean termination if possible (e.g., `taskkill /PID <pid>` if `child.kill()` is problematic, or investigate graceful shutdown signals if Syncthing supports them).
    - [ ] Clear the stored process handle.
    - [ ] Handle errors.
- [ ] Implement `is_syncthing_running()`:
    - [ ] Check the status of the stored `Child` handle (if Syncthing was started by this app).
    - [ ] If no `Child` handle exists (or to confirm status), attempt to detect if `syncthing.exe` is running by querying system processes (e.g., based on configured executable path or a known process name).
    - [ ] Update an internal state variable reflecting Syncthing's status.

## Phase 4: System Tray Icon Implementation

- [ ] Initialize the system tray icon in `main.rs` using `tray-item`.
- [ ] Prepare two icons (e.g., `.ico` files for Windows):
    - [ ] `icon_syncthing_running.ico`
    - [ ] `icon_syncthing_stopped.ico`
- [ ] Embed icons into the binary (e.g., using `include_bytes!`) or load them from files packaged with the app.
- [ ] Implement a function to update the tray icon based on Syncthing's running status.
- [ ] Implement a function to update the tray icon's tooltip text (e.g., "Syncthing: Running", "Syncthing: Stopped").
- [ ] Set up the main application loop required by `tray-item` to keep the icon responsive.

## Phase 5: Tray Menu and Actions

- [ ] Design the right-click menu structure:
    - [ ] "Status: Running" / "Status: Stopped" (dynamic text, possibly non-clickable, or a disabled item)
    - [ ] "Start Syncthing" (enabled when Syncthing is stopped)
    - [ ] "Stop Syncthing" (enabled when Syncthing is running)
    - [ ] --- Separator ---
    - [ ] "Open Syncthing Web UI"
    - [ ] "Open App Configuration"
    - [ ] --- Separator ---
    - [ ] "Exit"
- [ ] Implement menu item creation and event handling.
- [ ] Link "Start Syncthing" menu item:
    - [ ] Call `start_syncthing()`.
    - [ ] On success, update icon, tooltip, and menu item states (disable Start, enable Stop).
- [ ] Link "Stop Syncthing" menu item:
    - [ ] Call `stop_syncthing()`.
    - [ ] On success, update icon, tooltip, and menu item states (disable Stop, enable Start).
- [ ] Link "Open Syncthing Web UI" menu item:
    - [ ] Read `syncthing_gui_url` from config.
    - [ ] Use `opener::open()` to launch the URL in the default browser.
    - [ ] Handle errors (e.g., URL not configured or invalid).
- [ ] Link "Open App Configuration" menu item:
    - [ ] Get the path to `configuration.json`.
    - [ ] Use `opener::open()` to open the file (should use the default editor for `.json` files).
    - [ ] Handle errors (e.g., file cannot be opened).
- [ ] Link "Exit" menu item:
    - [ ] Perform cleanup (e.g., decide if Syncthing should be stopped if it was started by the app – perhaps make this configurable).
    - [ ] Terminate the tray application.

## Phase 6: Application State and Event Loop

- [ ] Define a central application state (e.g., in an `Arc<Mutex<AppState>>` or using an event channel like `crossbeam-channel`).
    - [ ] Store current `Config`.
    - [ ] Store `Option<std::process::Child>` for the Syncthing process.
    - [ ] Store current Syncthing status (e.g., an enum `SyncthingStatus { Stopped, Starting, RunningManaged, RunningExternal, Stopping }`).
    - [ ] Store handles/IDs to menu items if they need to be dynamically enabled/disabled or have their text changed.
- [ ] Implement the main event loop that processes messages from `tray-item` (menu clicks) and potentially other sources (e.g., timers for status checks).
- [ ] Ensure UI updates (icon, tooltip, menu states) are consistently reflecting the application state.
- [ ] Periodically check Syncthing's status if it was started by the app:
    - [ ] If `child.try_wait()` indicates it has exited unexpectedly, update status and UI.

## Phase 7: Error Handling and User Feedback

- [ ] Use the `thiserror` crate to define specific error types for different parts of the application (config, process, UI).
- [ ] Propagate errors using `Result<T, AppError>`.
- [ ] Log errors using the `log` crate.
- [ ] For critical errors (e.g., `syncthing.exe` not found at configured path, failed to write default config), consider showing a simple native dialog box (may require `windows-rs` or a small dialog crate).

## Phase 8: Build and Packaging (Windows Specific)

- [ ] Create an `app.rc` file and use the `winres` crate (build script) to:
    - [ ] Embed an application icon for the `.exe` file.
    - [ ] Set version information and other metadata for the executable.
- [ ] Ensure `cargo build --release` produces a working executable.
- [ ] Test the application by running the `.exe` directly (not via `cargo run`).
- [ ] Document where `syncthing.exe` needs to be located or how it should be configured.
- [ ] Consider creating an installer (e.g., using Inno Setup, WiX Toolset) for easier distribution (optional, advanced).

## Phase 9: Refinements and Polish

- [ ] Thoroughly test all functionalities:
    - [ ] Starting/stopping Syncthing multiple times.
    - [ ] Opening web UI and config file.
    - [ ] Behavior when `configuration.json` is missing or malformed.
    - [ ] Behavior when `syncthing.exe` path is incorrect.
- [ ] Add comments and documentation to the code.
- [ ] Review and refactor code for clarity, efficiency, and robustness.
- [ ] Consider adding simple notifications (e.g., "Syncthing started", "Error: Syncthing not found") using a crate like `notify-rust` if desired, though this adds complexity.
- [ ] Ensure graceful shutdown: what happens to Syncthing if the tray app is closed or crashes? (Configurable: leave running vs. stop).
- [ ] Implement initial and ongoing status check for externally started Syncthing:
    - [ ] On Syncthingers startup, and periodically (or on demand via `is_syncthing_running`), check if `syncthing.exe` (as configured, or by common process name) is running, even if not started by this app.
    - [ ] If detected as running externally:
        - [ ] Update UI (icon, tooltip, menu states: "Start" disabled, "Stop" enabled).
        - [ ] The "Stop Syncthing" action should attempt to terminate this external process.
    - [ ] If an externally running Syncthing instance stops (or one managed by the app stops unexpectedly), Syncthingers should detect this and update its UI.

## Future Considerations (Optional)

- [ ] Auto-start Syncthing when the tray application launches (add a setting in `configuration.json`).
- [ ] Add functionality to make the tray application start with Windows (e.g., by creating a shortcut in the Startup folder or a registry key).
- [ ] More sophisticated Syncthing status detection (e.g., checking if Syncthing's web UI is responsive, if Syncthing provides a status API endpoint).
- [ ] A simple UI panel (beyond the tray menu) for viewing logs or editing configuration if deemed necessary.

This list should provide a solid roadmap. Good luck with your project!