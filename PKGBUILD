pkgname=gamefic
pkgver=0.1.0
pkgrel=1
pkgdesc="Otimização do computador para jogos no Linux (Tauri 2)"
arch=(x86_64)
url="https://github.com/SM-Projetos/Gamefic"
license=(GPL-3.0-only)
depends=(gtk3 webkit2gtk-4.1)
makedepends=(nodejs npm rust)
source=("$pkgname-$pkgver.tar.gz::https://github.com/SM-Projetos/Gamefic/releases/download/v0.1.0/Gamefic-0.1.0.tar.gz")
sha256sums=(dce2c00439e532d12cd7311906ccb38c3b76ef2139b84556353c1a03ad1e0fa3)

build() {
  cd "Gamefic-0.1.0"
  npm install --no-audit --no-fund
  npm run build
  cd src-tauri
  cargo build --release --locked
}

package() {
  cd "Gamefic-0.1.0"
  install -Dm755 src-tauri/target/release/gamefic "$pkgdir/usr/bin/gamefic"
  install -Dm644 src-tauri/icons/128x128.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/gamefic.png"
  install -Dm644 src-tauri/icons/32x32.png "$pkgdir/usr/share/icons/hicolor/32x32/apps/gamefic.png"
  cat > gamefic.desktop <<DESK
[Desktop Entry]
Type=Application
Name=Gamefic
Comment=Otimização do computador para jogos no Linux
Exec=gamefic %U
Icon=gamefic
Terminal=false
Categories=GNOME;GTK;Game;System;
DESK
  install -Dm644 gamefic.desktop "$pkgdir/usr/share/applications/gamefic.desktop"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/gamefic/LICENSE"
}
