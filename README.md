# Syncthingers

A lightweight, Rust-based singleton system tray application for Windows to manage a local Syncthing instance.

## Features
- **Singleton enforcement:** Only one instance can run at a time.
- **System tray UI:** Start and stop Syncthing, monitor status, open web UI, and access configuration from the tray.
- **Process management:** Start, stop, and monitor Syncthing, with detection of externally started instances.
- **Configurable:** Application settings in `configuration.json`. Automatically updates configuration files when new options are added. Settings described below.
- **Logging:** Log key events and errors to a file.
- **Robust error handling:** User feedback via logs and native dialogs for critical errors.
- **Windows-specific:** Uses `.ico` tray icons and embeds version info for distribution.
- **Future-ready:** Platform-independent API design for potential cross-platform support.
- **Command-line config creation:** Use `--create-config` to only create the default config file and exit.

## Features (continued)
- **User directory for configuration:** Settings are stored in the user's AppData directory on Windows (specifically AppData\Local\Syncthingers).
- **Configuration path override:** Use `--config-path` argument to specify a custom location for configuration and logs.
- **Automatic migration:** Automatically migrates configuration from the executable directory if found.

## TODO Features
- **Syntching monitoring:** Monitor info like transfer speed from Synchting using web API.
- **Support external Syncthing instances:** Monitor Syncthing services from other computers and servers.

## Getting Started
1. **Clone the repository:**
   ```sh
   git clone <repo-url>
   cd syncthingers
   ```
2. **Install Rust:** [https://rustup.rs/](https://rustup.rs/)
3. **Build the app:**
   ```sh
   cargo build --release
   ```
4. **Configure:**
   - Edit `configuration.json` to set your Syncthing path, web UI URL, log level, and startup arguments.
   - Or run:
     ```sh
     target\release\syncthingers.exe --create-config
     ```
     to only create the default config file (if it doesn't exist) and exit.
5. **Run:**
   ```sh
   target\release\syncthingers.exe
   ```

## Command-Line Arguments

- `--log-level=<level>`: Set the initial log level for the app. Supported values: `off`, `error`, `warn`, `info`, `debug`. Example: `--log-level=debug`
- `--create-config`: Only create the default configuration file (if it doesn't exist) and exit. No tray or Syncthing process will be started.
- `--config-path=<path>`: Specify a custom path for configuration and log files. This can be a directory (where configuration.json and syncthingers.log will be created) or a specific file path for the configuration file. Example: `--config-path=C:\MyConfigs\syncthing`

You can combine these arguments as needed. For example:

```
syncthingers.exe --log-level=debug
```

This will start the app with debug-level logging.

## Directory Structure
```
assets/
  icons/           # .ico files for tray (running/stopped)
src/               # Rust source code
%LOCALAPPDATA%\Syncthingers\   # User configuration directory (Windows)
  configuration.json           # App configuration
  syncthingers.log             # Log file
```

> **Note:** In development mode, configuration.json and log files may be created in the current directory. In release mode, they are stored in the user's AppData directory.

## Configuration Example
```json
{
  "log_level": "info",
  "syncthing_path": "C:/Program Files/Syncthing/syncthing.exe",
  "web_ui_url": "http://localhost:8384",
  "startup_args": ["-no-browser"],
  "process_closure_behavior": "close_managed"
}
```

### Configuration Options

- **log_level**: Set the logging level (`off`, `error`, `warn`, `info`, `debug`)
- **syncthing_path**: Full path to the Syncthing executable
- **web_ui_url**: URL for the Syncthing web interface
- **startup_args**: Command line arguments passed to Syncthing when starting
- **process_closure_behavior**: Controls what happens to Syncthing processes when the app exits:
  - `"close_all"`: Closes all Syncthing processes (both managed and external)
  - `"close_managed"`: Only closes processes started by this app (default)
  - `"dont_close"`: Leaves all Syncthing processes running

## Development
- See `TODO.md` for a step-by-step plan and best practices.
- The codebase is organized for clarity, maintainability, and future cross-platform support.

## License
MIT
