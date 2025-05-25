# User Configuration Directory

Syncthingers now stores configuration and log files in the user's home directory to follow platform conventions and allow for better integration with the operating system.

## Overview

On Windows, configuration files are stored in:
```
%LOCALAPPDATA%\Syncthingers\
```

This directory contains:
- `configuration.json` - The main configuration file
- `syncthingers.log` - The application log file

## Features

### Automatic directory creation

The application automatically creates the necessary directories on startup if they don't exist.

### Configuration migration

When launched for the first time after upgrading to this version, Syncthingers will check for an existing configuration file in the executable directory. If found, it will copy this file to the user's configuration directory to ensure a smooth transition.

### Configuration path override

You can override the default configuration path by using the `--config-path` command-line argument:

```
syncthingers.exe --config-path=C:\MyConfig\syncthingers
```

This allows you to:
- Share configuration between multiple users
- Keep configuration on portable drives
- Use different configurations for different purposes

### Path handling

The application intelligently handles different path scenarios:

1. If `--config-path` specifies a directory, both configuration and log files will be created in that directory
2. If `--config-path` specifies a file, it will be used as the configuration file and logs will go to the same directory
3. If the specified path doesn't exist, it will be created (for directories)

## Implementation Details

The feature is implemented using the [dirs](https://crates.io/crates/dirs) crate, which provides a platform-independent way to locate user directories across different operating systems. This makes the code more maintainable and paves the way for future cross-platform support.

### Key components:

- `app_dirs.rs` - Core module handling all directory-related functionality
- Directory migration logic to support upgrading from previous versions
- Path resolution for configuration and log files
- Error handling for missing or inaccessible directories

## Fallback Mechanism

For backward compatibility and robustness, the application includes fallback mechanisms:

1. If the user directory cannot be accessed, fallback to the executable directory
2. If no configuration can be found/created, show an appropriate error message

This ensures a smooth experience for all users regardless of their system setup.
