# Rust GTK4 Rivalcfg Tray App

This is a Rust application using GTK4 for the GUI. It interacts with the rivalcfg CLI to control SteelSeries mice, and provides a system tray icon that represents the mouse battery level. The project is designed for easy Flatpak packaging and maximum compatibility with Wayland-based desktop environments and window managers.

## Features
- GTK4 GUI (Wayland-friendly)
- System tray icon shows battery level
- Interacts with rivalcfg CLI

## Requirements
- Rust (latest stable)
- GTK4 development libraries
- rivalcfg (installed and in PATH)

## Usage from source
1. Install dependencies: `cargo build`
2. Run the application: `cargo run`

## Roadmap
- Internal DB to remember and populate last known settings.
- Extend the GUI as needed for more mouse controls.
- For the love of all that is beautiful, I need to make it look better.
