pkgname=conman-git
pkgver=0.0.0
pkgrel=1
pkgdesc=""
arch="x86_64"
url="https://github.com/zarstensen/conman"
license="MIT"
makedepends=(git cargo)
source=("$pkgname::git+$url")

pkgver() {
    cd "$pkgname"
    git describe --long --abbrev=7 | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}

build() {
    cd "$pkgname"
    cargo build --release
}

package() {
    cd "$pkgname"

    install -Dm755 target/release/conman "$pkgdir/usr/bin/"
    install -Dm755 target/release/conman-hook "$pkgdir/usr/bin/"

    install -Dm644 hooks/conman-install.hook "$pkgdir/usr/share/libalpm/hooks/"
    install -Dm644 hooks/conman-remove.hook "$pkgdir/usr/share/libalpm/hooks/"

    install -Dm644 completions/conman.bash "$pkgdir/usr/share/bash-completion/completions/conman"
    install -Dm644 completions/conman.fish "$pkgdir/usr/share/fish/vendor_completions.d/conman.fish"
    install -Dm644 completions/conman.zsh "$pkgdir/usr/share/zsh/site-functions/_conman"
}
