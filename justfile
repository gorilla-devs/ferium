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

# Build for Linux
build-linux:
    rm -f out/ferium-linux.zip
    mkdir -p out
    cargo build --target=x86_64-unknown-linux-musl --release
    zip -r out/ferium-linux.zip -j target/x86_64-unknown-linux-musl/release/ferium

# Build for Linux without a GUI
build-linux-nogui:
    rm -f out/ferium-linux-nogui.zip
    mkdir -p out
    cargo build --target=x86_64-unknown-linux-musl --release --no-default-features
    zip -r out/ferium-linux-nogui.zip -j target/x86_64-unknown-linux-musl/release/ferium

# Build for Linux ARM64
build-linux-arm64:
    rm -f out/ferium-linux-arm64.zip
    mkdir -p out
    CC_aarch64_unknown_linux_musl=clang-14 CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld cargo rustc --target=aarch64-unknown-linux-musl --release -- -Clink-self-contained=yes -Clinker=rust-lld
    zip -r out/ferium-linux-arm64.zip -j target/aarch64-unknown-linux-musl/release/ferium

# Build for Linux ARM64 without a GUI
build-linux-arm64-nogui:
    rm -f out/ferium-linux-arm64-nogui.zip
    mkdir -p out
    CC_aarch64_unknown_linux_musl=clang-14 CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld cargo rustc --target=aarch64-unknown-linux-musl --release --no-default-features -- -Clink-self-contained=yes -Clinker=rust-lld 
    zip -r out/ferium-linux-arm64-nogui.zip -j target/aarch64-unknown-linux-musl/release/ferium

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
