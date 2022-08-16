default: install-dev
set windows-powershell := true

# Build for macOS Intel
build-mac-intel:
    rm -f out/ferium-macos-x64.zip
    mkdir -p out
    cargo build --target=x86_64-apple-darwin --release
    zip -r out/ferium-macos-x64.zip -j target/x86_64-apple-darwin/release/ferium

# Build for macOS Apple Silicon
build-mac-arm:
    rm -f out/ferium-macos-arm.zip
    mkdir -p out
    cargo build --target=aarch64-apple-darwin --release
    zip -r out/ferium-macos-arm.zip -j target/aarch64-apple-darwin/release/ferium

# Build for Windows MSVC
build-win:
    if (Test-Path -Path ".\out\ferium-windows-msvc.zip") { Remove-Item -Path ".\out\ferium-windows-msvc.zip" }
    if (-Not (Test-Path -Path ".\out")) { New-Item -Name "out" -ItemType Directory }
    cargo build --target=x86_64-pc-windows-msvc --release
    Compress-Archive -Path "target\x86_64-pc-windows-msvc\release\ferium.exe" -DestinationPath "out\ferium-windows-msvc.zip"

# Build for Windows GNU (e.g. Cygwin, MinGW)
build-win-gnu:
    rm -f out/ferium-windows-gnu.zip
    mkdir -p out
    cargo build --target=x86_64-pc-windows-gnu --release
    zip -r out/ferium-windows-gnu.zip -j target/x86_64-pc-windows-gnu/release/ferium.exe

# Build for GNU Linux with a GTK backend
build-linux-gtk:
    rm -f out/ferium-linux-gnu-gtk.zip
    mkdir -p out
    cargo build --target=x86_64-unknown-linux-gnu --release
    zip -r out/ferium-linux-gnu-gtk.zip -j target/x86_64-unknown-linux-gnu/release/ferium

# Build for GNU Linux with an XDG backend
build-linux-xdg:
    rm -f out/ferium-linux-gnu-xdg.zip
    mkdir -p out
    cargo build --target=x86_64-unknown-linux-gnu --release --no-default-features --features xdg
    zip -r out/ferium-linux-gnu-xdg.zip -j target/x86_64-unknown-linux-gnu/release/ferium

# Build for GNU Linux without a GUI backend
build-linux-nogui:
    rm -f out/ferium-linux-gnu-nogui.zip
    mkdir -p out
    cargo build --target=x86_64-unknown-linux-gnu --release --no-default-features
    zip -r out/ferium-linux-gnu-nogui.zip -j target/x86_64-unknown-linux-gnu/release/ferium

# Build for GNU Linux ARM64 with a GTK backend
build-linux-arm64-gtk:
    rm -f out/ferium-linux-gnu-arm64-gtk.zip
    mkdir -p out
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --target=aarch64-unknown-linux-gnu --release
    zip -r out/ferium-linux-gnu-arm64-gtk.zip -j target/aarch64-unknown-linux-gnu/release/ferium

# Build for GNU Linux ARM64 with an XDG backend
build-linux-arm64-xdg:
    rm -f out/ferium-linux-gnu-arm64-xdg.zip
    mkdir -p out
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --target=aarch64-unknown-linux-gnu --release --no-default-features --features xdg
    zip -r out/ferium-linux-gnu-arm64-xdg.zip -j target/aarch64-unknown-linux-gnu/release/ferium

# Build for GNU Linux ARM64 without a GUI backend
build-linux-arm64-nogui:
    rm -f out/ferium-linux-gnu-arm64-nogui.zip
    mkdir -p out
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --target=aarch64-unknown-linux-gnu --release --no-default-features
    zip -r out/ferium-linux-gnu-arm64-nogui.zip -j target/aarch64-unknown-linux-gnu/release/ferium

# Run clippy lints
lint:
    cargo clippy --   \
        -D clippy::all \
        -D clippy::exit \
        -D clippy::perf  \
        -D clippy::cargo  \
        -D clippy::style   \
        -D clippy::nursery  \
        -D clippy::pedantic  \
        -D clippy::dbg_macro  \
        -D clippy::suspicious  \
        -D clippy::unwrap_used  \
        -D clippy::complexity    \
        -D clippy::create_dir     \
        -D clippy::correctness     \
        -W clippy::expect_used      \
        -A clippy::too-many-lines    \
        -A clippy::must-use-candidate \
        -A clippy::multiple-crate-versions \

# Install Ferium to cargo's binary folder
install:
    cargo install --force --path .

# Install Ferium to cargo's binary folder but debug
install-dev:
    cargo install --debug --force --path .

# Delete all build and test artefacts
clean:
    cargo clean
    rm -rf out
    rm -rf tests/mods
    rm -rf tests/md_modpack
    rm -rf tests/cf_modpack
    rm -rf tests/configs/running
