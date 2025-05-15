# Syncthingers

A lightweight system tray application for managing Syncthing on Windows.

## Description

Syncthingers is a Rust-based singleton system tray application that provides convenient management of a local Syncthing instance. It allows users to:

- Start and stop Syncthing directly from the system tray
- Monitor Syncthing's running status via the tray icon
- Open the Syncthing web UI with a single click
- Access and edit the application configuration
- Ensure only one instance runs at a time

## Features

- System tray integration with dynamic icons based on Syncthing status
- One-click access to Syncthing web UI
- Process management for Syncthing (start/stop)
- Simple configuration via JSON file
- Singleton pattern ensuring only one instance runs

## Requirements

- Windows OS
- Syncthing executable installed and configured

## Installation

Download the latest release from the releases page and run the installer.

## Configuration

The application creates a `configuration.json` file on first run with default settings. You can customize:

- Path to Syncthing executable
- Web UI URL
- Additional Syncthing startup arguments
- Logging settings

## License

This project is licensed under the MIT License - see the LICENSE file for details. 