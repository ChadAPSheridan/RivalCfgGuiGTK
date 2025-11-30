#!/usr/bin/env bash
# Dry-run packaging helper - generates deb control and rpm spec into a temp dir and prints them
set -euo pipefail
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONF="$REPO_DIR/.github/workflows/config.conf"
if [ ! -f "$CONF" ]; then
  echo "config.conf not found at $CONF" >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$CONF"

TMPDIR="${TMPDIR:-/tmp}/rivalcfg-dryrun-$$"
rm -rf "$TMPDIR"
mkdir -p "$TMPDIR"

# Prepare minimal pkg layout
mkdir -p "$TMPDIR/pkg/usr/share/applications"
mkdir -p "$TMPDIR/pkg/usr/share/metainfo"
mkdir -p "$TMPDIR/pkg/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$TMPDIR/pkg/usr/share/icons/hicolor/scalable/apps"

cp -v "$REPO_DIR/rivalcfg-tray.desktop" "$TMPDIR/pkg/usr/share/applications/" || true
cp -v "$REPO_DIR/io.github.chadapsheridan.rivalcfgtray.appdata.xml" "$TMPDIR/pkg/usr/share/metainfo/" || true
cp -v "$REPO_DIR/icons/app_icon.png" "$TMPDIR/pkg/usr/share/icons/hicolor/256x256/apps/" || true

if compgen -G "$REPO_DIR/icons/*.svg" > /dev/null; then
  for f in "$REPO_DIR"/icons/*.svg; do
    cp -v "$f" "$TMPDIR/pkg/usr/share/icons/hicolor/scalable/apps/" || true
    base=$(basename "$f")
    size=$(echo "$base" | sed -n 's/[^0-9]*\([0-9][0-9]*\).*/\1/p')
    if [ -n "$size" ] 2>/dev/null; then
      mkdir -p "$TMPDIR/pkg/usr/share/icons/hicolor/${size}x${size}/apps"
      cp -v "$f" "$TMPDIR/pkg/usr/share/icons/hicolor/${size}x${size}/apps/" || true
    fi
  done
fi

DEB_RSVG=${DEB_RSVG:-librsvg2-bin}
RPM_RSVG=${RPM_RSVG:-librsvg2-tools}
MAINTAINER_VAL=${MAINTAINER:-"Chad Sheridan <chad.sheridan@cysec.ca>"}
PKGVER_VAL=${PKGVER:-"0.0.0"}

mkdir -p "$TMPDIR/debian-pkg/DEBIAN"
cp -r "$TMPDIR/pkg/usr" "$TMPDIR/debian-pkg/" || true
cat > "$TMPDIR/debian-pkg/DEBIAN/control" <<EOF
Package: rivalcfg-tray
Version: $PKGVER_VAL
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-3-0, libayatana-appindicator3-1, ${DEB_RSVG}, python3-pip
Recommends: python3-rivalcfg
Maintainer: $MAINTAINER_VAL
Description: System tray application for SteelSeries mouse configuration
 RivalCfg Tray is a system tray application that provides easy access to
 SteelSeries mouse configuration. It displays battery status and allows
 quick access to mouse settings directly from your system tray.
 .
 Note: rivalcfg may be installed as a pipx package. Run: sudo pipx install rivalcfg
EOF

cat > "$TMPDIR/debian-pkg/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
EOF
chmod 755 "$TMPDIR/debian-pkg/DEBIAN/postinst"

mkdir -p "$TMPDIR/rpmbuild/SPECS"
cat > "$TMPDIR/rpmbuild/SPECS/rivalcfg-tray.spec" <<EOF
Name:           rivalcfg-tray
Version:        $PKGVER_VAL
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

cp -r $TMPDIR/pkg/* \$RPM_BUILD_ROOT/

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
* $(date +'%a %b %d %Y') $MAINTAINER_VAL - $PKGVER_VAL-1
- Release $PKGVER_VAL

EOF

# Print outputs
echo
echo "==== Generated DEBIAN/control ===="
cat "$TMPDIR/debian-pkg/DEBIAN/control"

echo
echo "==== Generated RPMSPEC (temp) ===="
cat "$TMPDIR/rpmbuild/SPECS/rivalcfg-tray.spec"

echo
echo "Dry-run files are under: $TMPDIR"
