# Rust GTK Rivalcfg Tray App

This is a Rust application using GTK for the GUI and tray-icon for the system tray. It interacts with the rivalcfg CLI to control SteelSeries mice, and provides a system tray icon that represents the mouse battery level. The project is designed for easy Flatpak packaging and maximum compatibility with Wayland-based desktop environments and window managers.

## ⚠️ COSMIC Desktop Users

**As of version 1.2.0, this application uses tray-icon instead of libappindicator for better COSMIC desktop compatibility.** If you're upgrading from an older version, the new implementation should provide improved tray icon rendering across different desktop environments including COSMIC.

For historical information about the AppIndicator implementation, see [COSMIC_COMPATIBILITY.md](COSMIC_COMPATIBILITY.md).

## Features

- GTK GUI (Wayland-friendly)
- System tray icon shows battery level (using tray-icon library)
- Interacts with rivalcfg CLI

## Requirements

- Rust (latest stable)
- GTK development libraries
- rivalcfg (installed and in PATH)

## Usage from source

1. Install dependencies: `cargo build`

On Debian/Ubuntu you may need the GTK development package and friends:

```bash
sudo apt install libgtk-3-dev librsvg2-bin python3-pip pipx
```

On Fedora/RHEL the package names are typically:

```bash
sudo dnf install gtk3-devel librsvg2-tools python3-pip
```

1. Run the application: `cargo run`

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
sudo apt install libgtk-3-0 librsvg2-bin python3-pip pipx
sudo pipx install rivalcfg
```

Download the `.deb` package from the assets below, then install with dependency resolution:

```bash
sudo apt install ./rivalcfg-tray_${{ steps.config.outputs.VERSION }}_amd64.deb
```

**Fedora/RHEL:**
Install dependencies first:

```bash
sudo dnf install gtk3 librsvg2-tools python3-pip
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
- `librsvg` (or `librsvg2-bin`/`librsvg2-tools`) - For SVG to PNG conversion (provides `rsvg-convert`)
- GTK libraries

## Roadmap

- Extend the GUI as needed for more mouse controls.
- For the love of all that is beautiful, I need to make it look better.

## Packaging & release

This repository contains helper scripts and a GitHub Actions workflow to build release artifacts and distribution packages. Two important files are included in the repo:

- `build-and-prep.sh` — a local/CI helper script that builds the release binary and prepares a `pkg/` directory with installed files (binary, desktop entry, icons, appdata). This script is invoked by CI and can be run locally when preparing a release tarball.
- `.github/workflows/package-and-release.yaml` — GitHub Actions workflow that automates building, creating a source tarball, and generating DEB/RPM/PKGBUILD packages as release assets.

rsvg-convert dependency

The runtime pipeline in this project uses `rsvg-convert` to render SVG icons to PNG files for the tray indicator. That executable is provided by different packages on different distributions, so packaging should declare the appropriate runtime dependency for the target platform:

- Arch Linux (PKGBUILD): `librsvg` — add `librsvg` to `depends` (the `PKGBUILD` already does this).
- Debian/Ubuntu (.deb): `librsvg2-bin` — add `librsvg2-bin` to the package `Depends` (or use `cargo-deb` metadata to add it automatically).
- Fedora/RHEL (.rpm): `librsvg2-tools` — add `Requires: librsvg2-tools` to the spec.

When creating packages, make sure the packaging step in `package-and-release.yaml` or `build-and-prep.sh` copies the assembled `pkg/` into the appropriate package layout and that the generated package control files include the distribution-appropriate dependency on the provider of `rsvg-convert`.

If you want, I can update the CI YAML and `build-and-prep.sh` to explicitly set the correct Depends/Requires for the `.deb` and `.rpm` steps (e.g., inject `librsvg2-bin` into the deb control file and `librsvg2-tools` into the RPM spec) so the built packages will automatically list rsvg-convert as a runtime dependency.
