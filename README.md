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

## Flatpak
The project is structured for easy Flatpak packaging. Flatpaks are the intended format for official releases.

### Building the Flatpak
1. First, build the Rust application:
   ```bash
   cargo build --release
   ```

2. Set up the third-party dependencies:
   ```bash
   mkdir -p third_party
   cd third_party
   git clone https://github.com/flozz/rivalcfg.git
   cd ..
   ```

3. Build the Flatpak:
   ```bash
   flatpak-builder build-dir rivalcfg-tray.flatpak.yaml --force-clean
   ```

The build process will:
- Use the pre-built Rust binary from `target/release`
- Install the rivalcfg Python package from the local `third_party` directory
- Package everything into a Flatpak

## Roadmap
- The tray icon updates to reflect the current battery level on a more dynamic scale (currently the 25% marks seem a touch confusing at a glance).
- Internal DB to remember and populate last known settings.
- Extend the GUI as needed for more mouse controls.
- For the love of all that is beautiful, I need to make it look better.
