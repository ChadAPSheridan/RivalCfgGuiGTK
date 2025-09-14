# Maintainer: ChadAPSheridan <your-email@example.com>

pkgname=rivalcfg-gui-gtk
pkgver=0.9.0  # Replace with the actual version
pkgrel=1
pkgdesc="A GTK-based GUI for Rivalcfg"
arch=('x86_64')
url="https://github.com/ChadAPSheridan/RivalCfgGuiGTK"
license=('GPL')
depends=('gtk3' 'libayatana-appindicator3' 'hidapi' 'rivalcfg')
makedepends=('cargo' 'rust')
source=("$pkgname-$pkgver.tar.gz::https://github.com/ChadAPSheridan/RivalCfgGuiGTK/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')  # Replace with the actual checksum

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 target/release/rivalcfg-tray "$pkgdir/usr/bin/rivalcfg-tray"
  install -Dm644 io.github.chadapsheridan.rivalcfgtray.appdata.xml "$pkgdir/usr/share/metainfo/io.github.chadapsheridan.rivalcfgtray.appdata.xml"
  install -Dm644 rivalcfg-tray.desktop "$pkgdir/usr/share/applications/rivalcfg-tray.desktop"
  install -Dm644 icons/app_icon.png "$pkgdir/usr/share/icons/hicolor/256x256/apps/rivalcfg-tray.png"
  for icon in icons/*.svg; do
    size=$(basename "$icon" | sed 's/[^0-9]*\([0-9]*\).*/\1/')
    install -Dm644 "$icon" "$pkgdir/usr/share/icons/hicolor/${size}x${size}/apps/$(basename "$icon" .svg).png"
  done
}