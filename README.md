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

## Installation

**Arch Linux (AUR):**
```bash
yay -S rivalcfg-tray
# or
paru -S rivalcfg-tray
```

**Debian/Ubuntu:**

Install dependencies first:
```bash
sudo apt install libgtk-3-0 libayatana-appindicator3-1 librsvg2-bin python3-pip
sudo pip3 install rivalcfg
```
Download the `.deb` package from the assets below, then install with dependency resolution:
```bash
sudo apt install ./rivalcfg-tray_${{ steps.config.outputs.VERSION }}_amd64.deb
```

**Fedora/RHEL:**
Install dependencies first:
```bash
sudo dnf install gtk3 libayatana-appindicator-gtk3 librsvg2-tools python3-pip
sudo pip3 install rivalcfg
```

Download the `.rpm` package from the assets below, then install with dependency resolution:
```bash
sudo dnf install ./rivalcfg-tray-${{ steps.config.outputs.VERSION }}-1.*.rpm
```

**From Source:**
Download the source tarball and build with cargo.

### Dependencies
- `rivalcfg` - SteelSeries mouse configuration tool
- `librsvg` (or `librsvg2-bin`/`librsvg2-tools`) - For SVG to PNG conversion
- GTK3 and AppIndicator libraries


## Roadmap
- Internal DB to remember and populate last known settings.
- Extend the GUI as needed for more mouse controls.
- For the love of all that is beautiful, I need to make it look better.
