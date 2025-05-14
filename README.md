 # Syncthingers: Syncthing Tray Controller

 Syncthingers is a lightweight Rust-based Windows system tray application designed to manage your Syncthing instance. It allows you to easily start and stop `syncthing.exe`, monitor its status via the tray icon, and quickly access its web UI and your app's configuration.

## Features

*   **System Tray Icon:** Resides in the Windows system tray for easy access.
*   **Status Indication:** Tray icon visually changes to indicate whether Syncthing is running or stopped.
*   **Process Control:**
    *   Start Syncthing.
    *   Stop Syncthing.
*   **Quick Access Menu (Right-Click):**
    *   Start/Stop Syncthing.
    *   Open Syncthing Web UI in your default browser.
    *   Open the Syncthinger `configuration.json` file in your default editor.
    *   Exit the application.
*   **Configuration:** Uses a simple `configuration.json` file to manage paths and URLs.

## Prerequisites

*   **Rust:** Ensure you have Rust installed. You can get it from rustup.rs.
*   **Syncthing:** You need to have `syncthing.exe` downloaded and accessible on your system. You will configure its path in `configuration.json`.

## Building

1.  **Navigate to the project directory if you haven't already.**
2.  **Build the project:**
    *   For development:
        ```bash
        cargo build
        ```
    *   For a release version (optimized, no console window):
        ```bash
        cargo build --release
        ```
    The executable will be located in `target/debug/syncthinger.exe` or `target/release/syncthinger.exe`.

## Running

1. After building, run the executable (`syncthingers.exe`).
2.  On the first run, or if `configuration.json` is not found, a default one might be created (this functionality is planned). You will need to edit this file to point to your `syncthing.exe` and its web UI.

## Configuration

Syncthinger uses a `configuration.json` file to store its settings. This file typically includes:

*   `syncthing_executable_path`: Full path to your `syncthing.exe`.
*   `syncthing_gui_url`: URL for the Syncthing web interface (e.g., `http://127.0.0.1:8384`).
*   (Other settings as they are implemented)

The application will look for this file (e.g., alongside the executable or in a standard application data folder - exact location TBD).

## Development Roadmap

For a detailed list of planned features, ongoing tasks, and future considerations, please see the TODO.md file.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is usin MIT License