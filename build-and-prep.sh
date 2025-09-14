#! /bin/bash
# This script builds and prepares the RivalCfgGuiGTK project for packaging.
# It is intended to be used in a CI/CD pipeline or local development environment.
set -e
# Load configuration
source .github/workflows/config.conf
# Update instances of {{VERSION}} in files
sed -i "s/{{VERSION}}/$PKGVER/g" PKGBUILD io.github.chadapsheridan.rivalcfgtray.appdata.xml
# Update instances of {{BUILD_DATE}} in files
BUILD_DATE=$(date +%Y-%m-%d)
sed -i "s/{{BUILD_DATE}}/$BUILD_DATE/g" io.github.chadapsheridan.rivalcfgtray.appdata.xml
# Clean previous build artifacts
rm -rf target pkg

# Build the project
echo "Building $PKGNAME version $PKGVER..."
cargo build --release
# Prepare the package directory
echo "Preparing package directory..."
mkdir -p pkg/usr/bin
mkdir -p pkg/usr/share/applications
mkdir -p pkg/usr/share/icons/hicolor/256x256/apps
mkdir -p pkg/usr/share/metainfo
# Install the binary
install -Dm755 target/release/rivalcfg-tray pkg/usr/bin/rivalcfg-tray
# Install the desktop entry
install -Dm644 rivalcfg-tray.desktop pkg/usr/share/applications/rivalcfg-tray.desktop
# Install the appdata file
install -Dm644 io.github.chadapsheridan.rivalcfgtray.appdata.xml pkg/usr/share/metainfo/io.github.chadapsheridan.rivalcfgtray.appdata.xml
# Install the icons
install -Dm644 icons/app_icon.png pkg/usr/share/icons/hicolor/256x256/apps/rivalcfg-tray.png
# Also install a copy named to match the desktop/appdata id so the Desktop Icon= resolves
install -Dm644 icons/app_icon.png pkg/usr/share/icons/hicolor/256x256/apps/io.github.chadapsheridan.rivalcfgtray.png

# Install SVG icons into both scalable and sized directories
mkdir -p pkg/usr/share/icons/hicolor/scalable/apps
for icon in icons/*.svg; do
  base=$(basename "$icon")
  # Install as scalable SVG so icon themes can pick them
  install -Dm644 "$icon" "pkg/usr/share/icons/hicolor/scalable/apps/${base}"
  
  # Determine numeric size if present, e.g. battery-50.svg -> 50
  size=$(echo "$base" | sed -n 's/[^0-9]*\([0-9][0-9]*\).*/\1/p')
  if [ -n "$size" ] && [ "$size" -gt 0 ] && [ "$size" -le 512 ]; then
    mkdir -p "pkg/usr/share/icons/hicolor/${size}x${size}/apps"
    install -Dm644 "$icon" "pkg/usr/share/icons/hicolor/${size}x${size}/apps/${base}"
  fi
done
echo "Package directory prepared at pkg/"
echo "Build and preparation complete."