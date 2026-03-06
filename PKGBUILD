# Maintainer: Sebastien Rousseau <hello@sebastien.sh>
pkgname=rust-rlg
pkgver=0.0.7
pkgrel=1
pkgdesc="Brutalist, lock-free observability engine with AI-native telemetry support"
arch=('x86_64' 'aarch64')
url="https://github.com/sebastienrousseau/rlg"
license=('MIT' 'Apache')
depends=('systemd-libs' 'gcc-libs')
makedepends=('rust' 'cargo')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/sebastienrousseau/rlg/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "rlg-${pkgver}"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "rlg-${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    cargo build --frozen --release --all-features
}

check() {
    cd "rlg-${pkgver}"
    cargo test --frozen --all-features
}

package() {
    cd "rlg-${pkgver}"
    install -Dm755 "target/release/rlg" "${pkgdir}/usr/bin/rlg"
    install -Dm644 "README.md" "${pkgdir}/usr/share/doc/${pkgname}/README.md"
    install -Dm644 "LICENSE-MIT" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE-MIT"
    install -Dm644 "LICENSE-APACHE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE-APACHE"
}
