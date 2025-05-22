# Configuration Update Feature

## Overview

The Syncthingers app now supports automatic updating of configuration files when the application structure changes. When new configuration options are added to the application, existing configuration files will be automatically updated to include these new options with their default values.

## How It Works

1. When loading a configuration file, the app first tries to deserialize it directly into the current `Config` structure.
2. If deserialization fails (possibly due to missing fields), the app:
   - Parses the existing config into a generic JSON structure
   - Creates a new default configuration with all current fields
   - Merges the existing values with the default configuration
   - Writes the updated configuration back to the file

This ensures that:
- User-set values are preserved
- Missing fields are added with sensible defaults
- The configuration file is always compatible with the current version of the app

## Example

If a user has an older configuration file:

```json
{
  "log_level": "info",
  "syncthing_path": "C:/Program Files/Syncthing/syncthing.exe",
  "web_ui_url": "http://localhost:8384",
  "startup_args": ["-no-browser"]
}
```

And a new option `process_closure_behavior` is added to the app, after loading the app will automatically update the configuration to:

```json
{
  "log_level": "info",
  "syncthing_path": "C:/Program Files/Syncthing/syncthing.exe",
  "web_ui_url": "http://localhost:8384",
  "startup_args": ["-no-browser"],
  "process_closure_behavior": "close_managed"
}
```

## Implemented Features

1. **Automatic detection of missing fields**: The app detects when configuration fields are missing.
2. **Default value population**: Missing fields are populated with sensible default values.
3. **File update**: The updated configuration is written back to the configuration file.
4. **Transparent to users**: The process happens automatically without requiring user intervention.

## Benefits

This feature ensures a smooth upgrade experience for users:
- No configuration errors when updating to new versions
- No need for users to manually update their configuration files
- Default values provide a safe fallback for new options
- Configuration files remain backward and forward compatible
