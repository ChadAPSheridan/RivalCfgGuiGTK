# Changelog

All notable changes to this project will be documented in this file.

## [1.0.2] - 2026-01-11

### Changed

- **COSMIC Desktop Compatibility**: Updated icon handling to use proper StatusNotifier icon theme path approach
  - Icons are now stored in `$XDG_RUNTIME_DIR/rivalcfg-tray/` instead of `/tmp`
  - Uses `set_icon_theme_path()` with icon name only (following StatusNotifier spec)
  - Matches pattern used by other working tray applications (Steam, Sunshine)
  
### Added

- COSMIC Desktop compatibility documentation ([COSMIC_COMPATIBILITY.md](COSMIC_COMPATIBILITY.md))
- Diagnostic script for troubleshooting tray icon issues ([scripts/cosmic-tray-diagnostics.sh](scripts/cosmic-tray-diagnostics.sh))
- Cleanup function now handles both temp and runtime directories

### Fixed

- Icon path handling to better comply with StatusNotifier specification
- Runtime directory cleanup on application exit

### Known Issues

- Tray icon may still not be visible in COSMIC Desktop 1.0.1 due to upstream rendering issues
- The application works correctly, but COSMIC's cosmic-applet-status-area may not render StatusNotifier icons properly

## [1.0.1] - Previous Release

### Features

- GTK GUI with Wayland support
- System tray icon with battery level indication
- SteelSeries mouse configuration interface
- Custom icon color themes (light/dark/custom)
- Polling rate, sensitivity, sleep timer configuration
- Battery status monitoring

### Dependencies

- gtk 0.18
- gio 0.18
- appindicator3 0.3
- gdk-pixbuf 0.18
- librsvg (runtime dependency for SVG to PNG conversion)
