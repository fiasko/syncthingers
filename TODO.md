# TODO List: Syncthing Tray Controller

This document outlines the development plan for a Rust-based Windows system tray application to manage Syncthing.

## Phase 1: Project Setup & Core Dependencies

- [ ] Initialize a new Rust project: `cargo new syncthing_tray_controller --bin`
- [ ] Add `#![windows_subsystem = "windows"]` to `main.rs` to hide the console window on launch.
- [ ] Research and choose a system tray library suitable for Windows:
    - [ ] Evaluate `tray-item` (cross-platform, might be simpler to start).
    - [ ] Evaluate `systray` (another cross-platform option).
    - [ ] Consider `windows-rs` for direct WinAPI calls if more fine-grained control or specific Windows features are needed.
- [ ] Add chosen system tray library to `Cargo.toml`.
- [ ] Add `serde` and `serde_json` for configuration file handling: `cargo add serde serde_json --features serde/derive`
- [ ] Add `opener` crate for opening web pages and files: `cargo add opener`
- [ ] Add `thiserror` for custom error types: `cargo add thiserror`
- [ ] Add `log` and a logger implementation (e.g., `env_logger` or `simplelog`): `cargo add log env_logger`
- [ ] Set up basic logging configuration.

## Phase 2: Configuration Management (`configuration.json`)

- [ ] Define the structure for `configuration.json`:
    - [ ] `syncthing_executable_path`: Full path to `syncthing.exe`.
    - [ ] `syncthing_gui_url`: URL for Syncthing's web UI (e.g., `http://127.0.0.1:8384`).
    - [ ] `syncthing_startup_args`: (Optional) Array of strings for arguments to pass to `syncthing.exe` on start.
    - [ ] `config_file_path`: Path to this `configuration.json` file (for "Open Configuration" feature, could be self-determined).
- [ ] Implement a `Config` struct with `serde::Deserialize` and `serde::Serialize`.
- [ ] Implement a function to load `configuration.json`:
    - [ ] Determine path (e.g., alongside the executable, or in `%APPDATA%/<AppName>`).
    - [ ] If not found, create a default `configuration.json` with placeholder values.
    - [ ] Read and parse the file.
    - [ ] Handle I/O and parsing errors gracefully.
- [ ] Implement a function to save/create a default `configuration.json`.
- [ ] Ensure the application can locate its own configuration file for the "Open Configuration" feature.

## Phase 3: Syncthing Process Management

- [ ] Create a module/struct for managing the Syncthing process (e.g., `SyncthingManager`).
- [ ] Implement `start_syncthing()`:
    - [ ] Read `syncthing_executable_path` and `syncthing_startup_args` from config.
    - [ ] Use `std::process::Command` to launch `syncthing.exe`.
    - [ ] Ensure it runs in the background (e.g., detached, no new console window if Syncthing itself doesn't handle this).
    - [ ] Store the `std::process::Child` handle to monitor and stop the process.
    - [ ] Handle errors (e.g., executable not found, failed to start).
- [ ] Implement `stop_syncthing()`:
    - [ ] Use the stored `Child` handle to kill the process.
    - [ ] On Windows, ensure it's a clean termination if possible (e.g., `taskkill /PID <pid>` if `child.kill()` is problematic, or investigate graceful shutdown signals if Syncthing supports them).
    - [ ] Clear the stored process handle.
    - [ ] Handle errors.
- [ ] Implement `is_syncthing_running()`:
    - [ ] Check the status of the stored `Child` handle (e.g., using `try_wait()`).
    - [ ] If no handle exists (e.g., app just started), this might initially report "stopped" or attempt a more complex check (e.g., by process name, though this can be unreliable). For v1, primarily rely on the state managed by this app.
    - [ ] Update an internal state variable reflecting Syncthing's status.

## Phase 4: System Tray Icon Implementation

- [ ] Initialize the system tray icon in `main.rs` using the chosen library.
- [ ] Prepare two icons (e.g., `.ico` files for Windows):
    - [ ] `icon_syncthing_running.ico`
    - [ ] `icon_syncthing_stopped.ico`
- [ ] Embed icons into the binary (e.g., using `include_bytes!`) or load them from files packaged with the app.
- [ ] Implement a function to update the tray icon based on Syncthing's running status.
- [ ] Implement a function to update the tray icon's tooltip text (e.g., "Syncthing: Running", "Syncthing: Stopped").
- [ ] Set up the main application loop required by the tray library to keep the icon responsive.

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
    - [ ] Store current Syncthing status (e.g., an enum `SyncthingStatus { Running, Stopped, Starting, Stopping }`).
    - [ ] Store handles/IDs to menu items if they need to be dynamically enabled/disabled or have their text changed.
- [ ] Implement the main event loop that processes messages from the tray library (menu clicks) and potentially other sources (e.g., timers for status checks).
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
- [ ] Initial status check: When the tray app starts, if `syncthing.exe` is *already* running (not started by this app), how should it behave?
    - [ ] Option 1 (Simpler): Assume it's not running until "Start" is clicked.
    - [ ] Option 2 (Advanced): Try to detect an existing Syncthing process and reflect its status. This is harder to do reliably.

## Future Considerations (Optional)

- [ ] Auto-start Syncthing when the tray application launches (add a setting in `configuration.json`).
- [ ] Add functionality to make the tray application start with Windows (e.g., by creating a shortcut in the Startup folder or a registry key).
- [ ] More sophisticated Syncthing status detection (e.g., checking if Syncthing's web UI is responsive, if Syncthing provides a status API endpoint).
- [ ] A simple UI panel (beyond the tray menu) for viewing logs or editing configuration if deemed necessary.

This list should provide a solid roadmap. Good luck with your project!