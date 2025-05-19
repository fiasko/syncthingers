# Syncthingers

A lightweight, Rust-based singleton system tray application for Windows to manage a local Syncthing instance.

## Features
- **Singleton enforcement:** Only one instance can run at a time.
- **System tray UI:** Start/stop Syncthing, monitor status, open web UI, and access configuration from the tray.
- **Process management:** Start, stop, and monitor Syncthing, with detection of externally started instances.
- **Configurable:** Settings (log level, Syncthing path, web UI URL, startup args) in `configuration.json`.
- **Logging:** Log key events and errors to a file.
- **Robust error handling:** User feedback via logs and native dialogs for critical errors.
- **Windows-specific:** Uses `.ico` tray icons and embeds version info for distribution.
- **Future-ready:** Platform-independent API design for potential cross-platform support.
- **Command-line config creation:** Use `--create-config` to only create the default config file and exit.

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
configuration.json # App configuration
syncthingers.log   # Log file
```

## Configuration Example
```
{
  "log_level": "info",
  "syncthing_path": "C:/Program Files/Syncthing/syncthing.exe",
  "web_ui_url": "http://localhost:8384",
  "startup_args": ["-no-browser"]
}
```

## Development
- See `TODO.md` for a step-by-step plan and best practices.
- The codebase is organized for clarity, maintainability, and future cross-platform support.

## License
MIT
