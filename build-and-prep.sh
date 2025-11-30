#! /bin/bash
# This script builds and prepares the RivalCfgGuiGTK project for packaging.
# It is intended to be used in a CI/CD pipeline or local development environment.
set -e
# Load configuration
source .github/workflows/config.conf

# Helper: check prerequisites and optionally install
ensure_prereqs() {
  local auto_install=false
  if [ "$1" = "--yes" ]; then
    auto_install=true
  fi

  # Commands we need
  local need=(cargo rustc install rsvg-convert dpkg-deb rpmbuild gtk-update-icon-cache)
  # But cargo and rustc are build-time; we only check for packaging helper commands here
  local check_cmds=(rsvg-convert dpkg-deb rpmbuild gtk-update-icon-cache)

  # Detect package manager
  local pkg_mgr=""
  if command -v apt-get >/dev/null 2>&1; then
    pkg_mgr=apt
  elif command -v dnf >/dev/null 2>&1; then
    pkg_mgr=dnf
  elif command -v pacman >/dev/null 2>&1; then
    pkg_mgr=pacman
  fi

  local missing=()
  for c in "${check_cmds[@]}"; do
    if ! command -v "$c" >/dev/null 2>&1; then
      missing+=("$c")
    fi
  done

  if [ ${#missing[@]} -eq 0 ]; then
    echo "All packaging prerequisites present."
    return 0
  fi

  echo "Missing packaging prerequisites: ${missing[*]}"
  if [ "$auto_install" = true ] && [ -n "$pkg_mgr" ]; then
    echo "Attempting to install missing packages via $pkg_mgr..."
    case $pkg_mgr in
      apt)
        sudo apt-get update
        sudo apt-get install -y librsvg2-bin dpkg-dev rpm-build gtk-update-icon-cache || true
        ;;
      dnf)
        sudo dnf install -y librsvg2-tools rpm-build gtk-update-icon-cache || true
        ;;
      pacman)
        sudo pacman -Sy --noconfirm librsvg rpm-build || true
        ;;
    esac
    return 0
  fi

  echo "To install prerequisites, run one of the following (as appropriate for your distro):"
  echo "  Debian/Ubuntu: sudo apt-get install librsvg2-bin dpkg-dev rpm-build gtk-update-icon-cache"
  echo "  Fedora/RHEL: sudo dnf install librsvg2-tools rpm-build gtk-update-icon-cache"
  echo "  Arch: sudo pacman -S librsvg rpm-build"
  echo "Or re-run this script with --yes to attempt automatic install (will use detected package manager)."
  return 1
}

# Run prereq check unless explicitly skipped
if [ "${SKIP_PREREQ_CHECK:-}" != "1" ]; then
  if ! ensure_prereqs "$1"; then
    echo "Prerequisite check failed. Aborting."
    exit 1
  fi
fi
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

# Packaging helpers: create .deb control and rpm spec for local builds if requested
# Allow overriding provider package names via environment variables
DEB_RSVG=${DEB_RSVG:-librsvg2-bin}
RPM_RSVG=${RPM_RSVG:-librsvg2-tools}

echo "Preparing packaging metadata..."

# Create debian package structure and control file
mkdir -p debian-pkg/DEBIAN
mkdir -p debian-pkg/usr
cp -r pkg/usr/* debian-pkg/usr/

cat > debian-pkg/DEBIAN/control << EOF
Package: rivalcfg-tray
Version: $PKGVER
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-3-0, libayatana-appindicator3-1, ${DEB_RSVG}, python3-pip
Recommends: python3-rivalcfg
Maintainer: $MAINTAINER
Description: System tray application for SteelSeries mouse configuration
 RivalCfg Tray is a system tray application that provides easy access to
 SteelSeries mouse configuration. It displays battery status and allows
 quick access to mouse settings directly from your system tray.
 .
 Note: rivalcfg may be installed as a pipx package. Run: sudo pipx install rivalcfg
EOF

cat > debian-pkg/DEBIAN/postinst << 'EOF'
#!/bin/bash
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
EOF
chmod 755 debian-pkg/DEBIAN/postinst

echo "Debian package metadata prepared (debian-pkg/)."

# Create RPM spec file for local rpmbuild
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cat > ~/rpmbuild/SPECS/rivalcfg-tray.spec << EOF
Name:           rivalcfg-tray
Version:        $PKGVER
Release:        1%{?dist}
Summary:        System tray application for SteelSeries mouse configuration

License:        GPLv3+
URL:            https://github.com/ChadAPSheridan/RivalCfgGuiGTK

Requires:       gtk3
Requires:       libayatana-appindicator-gtk3
Requires:       ${RPM_RSVG}
Requires:       rivalcfg

%description
RivalCfg Tray is a system tray application that provides easy access to
SteelSeries mouse configuration. It displays battery status and allows
quick access to mouse settings directly from your system tray.

%prep
# No prep needed - using pre-built binary

%build
# No build needed - using pre-built binary

%install
rm -rf \$RPM_BUILD_ROOT
mkdir -p \$RPM_BUILD_ROOT/usr/bin
mkdir -p \$RPM_BUILD_ROOT/usr/share/applications
mkdir -p \$RPM_BUILD_ROOT/usr/share/metainfo
mkdir -p \$RPM_BUILD_ROOT/usr/share/icons/hicolor/256x256/apps
mkdir -p \$RPM_BUILD_ROOT/usr/share/icons/hicolor/scalable/apps

cp -r $(pwd)/pkg/* \$RPM_BUILD_ROOT/

%post
/bin/touch --no-create /usr/share/icons/hicolor &>/dev/null || :

%postun
if [ \$1 -eq 0 ] ; then
    /bin/touch --no-create /usr/share/icons/hicolor &>/dev/null
    /usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor &>/dev/null || :
fi

%posttrans
/usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor &>/dev/null || :

%files
/usr/bin/rivalcfg-tray
/usr/share/applications/rivalcfg-tray.desktop
/usr/share/metainfo/io.github.chadapsheridan.rivalcfgtray.appdata.xml
/usr/share/icons/hicolor/*/apps/*

%changelog
* $(date +'%a %b %d %Y') $MAINTAINER - $PKGVER-1
- Release $PKGVER

EOF

echo "RPM spec prepared (~/rpmbuild/SPECS/rivalcfg-tray.spec)."

# Optionally build .deb if dpkg-deb available
if command -v dpkg-deb >/dev/null 2>&1; then
  echo "Building local DEB package..."
  dpkg-deb --build debian-pkg rivalcfg-tray_${PKGVER}_amd64.deb
  echo "Built deb: rivalcfg-tray_${PKGVER}_amd64.deb"
fi

# Optionally build RPM if rpmbuild available
if command -v rpmbuild >/dev/null 2>&1; then
  echo "Building local RPM package..."
  rpmbuild -ba ~/rpmbuild/SPECS/rivalcfg-tray.spec
  cp ~/rpmbuild/RPMS/x86_64/rivalcfg-tray-*.rpm ./ || true
  echo "RPM build complete (copied to workspace if present)."
fi