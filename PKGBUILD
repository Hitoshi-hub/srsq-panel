# Maintainer: Hitoshi <ruriupin67@gmail.com>
pkgname=srsq-panel
pkgver=0.3.2
options=('!debug')
pkgrel=1
pkgdesc="Custom modular Wayland status panel for Sway"
arch=('x86_64')
url="https://github.com/Hitoshi-hub/srsq-panel"
license=('MIT')
depends=(
    'gtk4'
    'gtk4-layer-shell'
    'glib2'
    'pango'
    'cairo'
)
makedepends=(
    'cargo'
    'git'
    'rust'
)
source=("$pkgname-$pkgver::git+$url.git#tag=v$pkgver") 
b2sums=('SKIP')

prepare() {
  cd "$pkgname-$pkgver"
  
  cargo fetch --locked --target "$(rustc -Vv | grep host | cut -f2 -d' ')"
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  
  cargo build --frozen --release --all-features
}

package() {
  cd "$pkgname-$pkgver"
  
  install -Dm0755 "target/release/srsq-panel" "$pkgdir/usr/bin/srsq-panel"
  
  if [ -d "resources" ]; then
    install -dm0755 "$pkgdir/usr/share/srsq-panel"
    cp -r resources/* "$pkgdir/usr/share/srsq-panel/"
  fi

}