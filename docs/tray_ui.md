# System Tray UI Documentation

This document describes the system tray UI implementation in the Syncthingers application, specifically the `tray_ui.rs` module.

**Last Updated: May 30, 2025**

## Overview

The system tray UI provides the primary user interface for Syncthingers, allowing users to control Syncthing processes and access application features through a context menu. The implementation uses the `tray-item` crate for cross-platform system tray functionality.

### Key Features

1. **Dynamic State Visualization**:
   - Green icon when Syncthing is running
   - Red icon when Syncthing is stopped
   - Real-time state updates through background monitoring

2. **Process State Monitoring**:
   - Polling-based monitoring with 2-second intervals
   - Automatic detection of external Syncthing processes
   - Thread-safe state synchronization between UI and process management

3. **Context Menu Actions**:
   - Start/Stop Syncthing with dynamic menu text
   - Open Syncthing Web UI in default browser
   - Open configuration file in default editor
   - Exit application with configurable process closure behavior

4. **Robust Error Handling**:
   - Comprehensive error propagation with custom error types
   - Graceful degradation when operations fail
   - Detailed logging for debugging and monitoring

5. **Thread Safety**:
   - Arc<Mutex<>> patterns for shared state access
   - Weak references to prevent circular dependencies
   - Background monitoring thread with proper cleanup

## Architecture

### Core Types

#### TrayState Enum
```rust
pub enum TrayState {
    Running,
    Stopped,
}
```
Represents the visual state of the system tray icon, mapped to Syncthing process status.

#### TrayMenuAction Enum
```rust
pub enum TrayMenuAction {
    StartStop,
    OpenWebUI,
    OpenConfig,
    Exit,
}
```
Defines the available actions in the tray context menu.

#### TrayUi Struct
The main system tray component containing:
- `tray: TrayItem` - The actual system tray item from tray-item crate
- `state: TrayState` - Current visual state
- `app_state: Arc<Mutex<AppState>>` - Shared application state

### Public API

#### `TrayUi::new(app_state: Arc<Mutex<AppState>>) -> Result<Arc<Mutex<Self>>, Box<dyn Error>>`
Creates a new TrayUi instance with:
- Initial state detection based on running Syncthing processes
- Red icon by default (indicating stopped state)
- Automatic background monitoring thread startup
- Thread-safe Arc<Mutex<>> wrapper for safe concurrent access

#### `set_state(&mut self, state: TrayState)`
Updates the internal tray state without refreshing the UI. Used by the monitoring thread to track state changes.

#### `setup_tray_menu(&mut self) -> Result<(), AppError>`
Initializes the tray menu by calling `recreate_tray_menu()`. This is the primary entry point for setting up the UI.

#### `recreate_tray_menu(&mut self) -> Result<(), AppError>`
Completely rebuilds the tray menu with:
- Updated icon based on current state (green for running, red for stopped)
- Dynamic menu text ("Start Syncthing" vs "Stop Syncthing")
- Fresh callback bindings to prevent stale references

#### `handle_menu_action_static(app_state: Arc<Mutex<AppState>>, action: TrayMenuAction) -> Result<(), AppError>`
Static method for processing menu actions. Required because tray callbacks cannot capture `&self` references.

### Private Implementation

#### State Detection and Monitoring

##### `detect_initial_state(app_state: &Arc<Mutex<AppState>>) -> Result<TrayState, Box<dyn Error>>`
Determines the initial tray state by checking if Syncthing is already running using `AppState::syncthing_running()`.

##### `start_monitoring_thread(tray_ui_ptr: Arc<Mutex<Self>>, app_state: Arc<Mutex<AppState>>) -> Result<(), Box<dyn Error>>`
Spawns a background thread that:
- Polls process state every 2 seconds
- Uses weak references to prevent circular dependencies
- Automatically exits when TrayUi is dropped
- Updates tray icon and menu when state changes

##### `get_current_process_state(app_state: &Arc<Mutex<AppState>>) -> (TrayState, String)`
Returns current process state and origin information:
- `(TrayState::Running, "started by app")` - Process launched by this application
- `(TrayState::Running, "external")` - External Syncthing process detected
- `(TrayState::Stopped, "not running")` - No Syncthing process found

##### `log_process_state(process_origin: &str)`
Logs process state changes with appropriate log levels for monitoring and debugging.

#### Menu Management

##### `add_menu_items(&self, tray: &mut TrayItem) -> Result<(), AppError>`
Adds all menu items in order:
1. Start/Stop Syncthing (dynamic text based on state)
2. Open Syncthing Web UI
3. Open Configuration
4. Exit

##### `add_menu_item(&self, tray: &mut TrayItem, label: &str, action: TrayMenuAction) -> Result<(), AppError>`
Helper method that:
- Creates closures with cloned app_state reference
- Binds actions to menu items
- Handles callback registration errors

##### `process_menu_action(state: &mut AppState, action: TrayMenuAction) -> Result<(), AppError>`
Processes specific menu actions:

**StartStop**: Toggles Syncthing state using `AppState::start_syncthing()` / `AppState::stop_syncthing()`

**OpenWebUI**: Opens the configured web UI URL using the `opener` crate

**OpenConfig**: 
- Locates config file using `AppDirs::config_file_path()`
- Validates file existence
- Opens in default editor using `Config::open_in_editor()`

**Exit**: 
- Calls `AppState::handle_exit_closure()` for proper cleanup
- Terminates application with `std::process::exit(0)`

## Process Integration

The tray UI integrates with the process management system through several key patterns:

### State Synchronization
- Uses `AppState::syncthing_running()` for consistent process state checking
- Relies on AppState's process cleanup logic during state queries
- Handles both app-launched and external Syncthing processes

### Thread Safety
- All shared state access through Arc<Mutex<>> patterns
- Weak references in monitoring thread prevent memory leaks
- Static menu action handlers avoid lifetime issues with callbacks

### Error Handling
- Custom `AppError::TrayUiError` variants for UI-specific errors
- Graceful degradation when operations fail
- Comprehensive logging for debugging

## Configuration Integration

The tray UI respects application configuration through:

### Web UI URL
Uses `config.web_ui_url` for the "Open Syncthing Web UI" action.

### Process Closure Behavior
Honors `config.process_closure_behavior` during application exit through `AppState::handle_exit_closure()`.

### Configuration File Access
Uses `AppDirs` to locate and open configuration files in the user's preferred editor.

## Testing Strategy

The test module in `tray_ui.rs` provides:

### Test Helpers
- `create_test_config()`: Creates minimal test configuration
- `dummy_app_dirs()`: Creates test app directories instance

### Test Coverage
- **Initial State Detection**: Tests the `detect_initial_state()` API pattern
- **Process State Queries**: Validates `get_current_process_state()` behavior
- **Menu Action Processing**: Template for testing menu actions (limited by `std::process::exit`)

### Testing Limitations
Some functionality requires mocking for full testing:
- System tray integration (requires display environment)
- Process exit behavior (calls `std::process::exit`)
- External process detection (requires system process access)
- File system operations (config file opening)

## Platform Considerations

### Cross-Platform Tray Support
Uses `tray-item` crate for Windows, macOS, and Linux compatibility.

### Icon Resources
References `syncthing_green` and `syncthing_red` icon resources that must be embedded in the application binary.

### Menu Behavior
Follows platform-specific tray menu conventions through the `tray-item` abstraction.

## Error Scenarios and Recovery

### Tray Creation Failures
If initial tray creation fails, the application returns an error during startup rather than continuing without UI.

### Menu Recreation Failures
Menu recreation errors are logged as warnings but don't crash the application. The monitoring thread continues operation.

### App State Lock Failures
Lock acquisition failures result in `AppError::TrayUiError` returns, allowing calling code to handle the error appropriately.

### Configuration File Access
Missing configuration files result in specific error messages to help users locate or create the required files.

## Performance Characteristics

### Polling Frequency
The monitoring thread polls every 2 seconds, balancing responsiveness with system resource usage.

### Memory Usage
Uses weak references in the monitoring thread to prevent memory leaks when the TrayUi is dropped.

### Thread Management
Single background thread per TrayUi instance with automatic cleanup when the instance is destroyed.

## Future Enhancements

### Testing Improvements
1. Mock framework integration for comprehensive testing
2. Headless testing capabilities for CI/CD environments
3. Integration tests with actual process lifecycle

### Feature Additions
1. Tooltip text updates based on process state
2. Menu item icons for better visual hierarchy
3. Keyboard shortcuts for menu actions
4. Multiple Syncthing instance support

### Performance Optimizations
1. Event-driven state updates instead of polling
2. Configurable polling intervals
3. Smart polling (faster when state is changing)
