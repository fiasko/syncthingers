# Process Management Documentation

This document describes the process management implementation in the Syncthingers application, specifically the `process.rs` module.

## Overview

The process management system in Syncthingers has been designed to provide process control for Syncthing instances. It uses the `sysinfo` crate for cross-platform process monitoring and control.

### Key Features

1. **Cross-Platform Process Management**:
   - Uses `sysinfo` crate for process detection, monitoring, and termination
   - Works consistently across different operating systems

2. **Process Tree Tracking**:
   - Tracks all Syncthing processes (parent and children)
   - Ensures complete shutdown of process trees
   - Prevents orphaned child processes

3. **External Process Detection**:
   - Can detect and manage Syncthing processes not started by the app
   - Provides capability to stop external Syncthing instances
   - Differentiates between app-managed and external processes

4. **Console Window Prevention**:
   - On Windows, prevents console windows from appearing when starting processes
   - Uses `CREATE_NO_WINDOW` flag for clean background operation

5. **Robust Error Handling**:
   - Comprehensive error messages and logging
   - Graceful handling of process state transitions
   - Test environment detection to prevent interference

## Implementation Details

### SyncthingProcess Struct

The core `SyncthingProcess` struct manages individual Syncthing instances:

```rust
pub struct SyncthingProcess {
    child: Option<Child>,           // Child process handle (for app-started processes)
    pub started_by_app: bool,       // Flag indicating if process was started by app
    pub syncthing_path: String,     // Path to Syncthing executable
    pub pid: Option<u32>,          // Main process ID
    tracked_pids: Vec<u32>,        // All tracked Syncthing process IDs
    system: System,                // sysinfo System instance for monitoring
}
```

### Public API

**Process Lifecycle Management:**
- `SyncthingProcess::new(path: &str)`: Creates a new process manager instance
- `start(&mut self, args: &[String])`: Starts a new Syncthing process with specified arguments
- `stop(&mut self)`: Stops the process and all tracked child processes
- `is_running(&mut self)`: Checks if the process is currently running

**Process Detection:**
- `detect_process(syncthing_path: &str, external_only: bool)`: Detects existing Syncthing processes
- `stop_external_syncthing_processes(syncthing_path: &str)`: Stops all external Syncthing processes

## Potential Improvements for the future

1. **Enhanced Process Tree Discovery**:
   ```rust
   fn discover_process_tree(&mut self, root_pid: u32) -> Vec<u32> {
       // Recursive discovery of all child processes
   }
   ```

2. **Graceful Shutdown**:
   ```rust
   fn graceful_stop(&mut self, timeout: Duration) -> io::Result<()> {
       // Attempt graceful shutdown before force termination
   }
   ```
