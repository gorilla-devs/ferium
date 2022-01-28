.SILENT: test
.DEFAULT_GOAL := install-dev

# This builds for macOS Intel and macOS Apple Silicon
build-mac:
	make test
# Remove previous builds
	rm -f out/ferium-macos-x64.zip out/ferium-macos-arm.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-apple-darwin --release
	cargo build --target=aarch64-apple-darwin --release
# Zip and move executables to out/
	zip -r out/ferium-macos-x64.zip -j target/x86_64-apple-darwin/release/ferium
	zip -r out/ferium-macos-arm.zip -j target/aarch64-apple-darwin/release/ferium

# This builds for Windows MSVC
build-win:
#	make test
# Remove previous build if there is one
	IF EXIST out\ferium-windows-msvc.zip DEL out\ferium-windows-msvc.zip
# Make builds output directory if it doesn't exist
	IF NOT EXIST out MKDIR out
# Build for target
	cargo build --target=x86_64-pc-windows-msvc --release
# Zip and move executable to out/
	PowerShell -Command Compress-Archive -Path "target\x86_64-pc-windows-msvc\release\ferium.exe" -DestinationPath "out\ferium-windows-msvc.zip"

# This builds for GNU Linux and GNU Windows (e.g. cygwin)
build-linux:
	make test
# Remove previous builds
	rm -f out/ferium-linux-gnu.zip out/ferium-windows-gnu.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-pc-windows-gnu --release
	cargo build --target=x86_64-unknown-linux-gnu --release
# Zip and move executables to out/
	zip -r out/ferium-linux-gnu.zip -j target/x86_64-unknown-linux-gnu/release/ferium
	zip -r out/ferium-windows-gnu.zip -j target/x86_64-pc-windows-gnu/release/ferium.exe

test:
	cargo clippy -- \
		-D clippy::all \
		-D clippy::cargo \
		-D clippy::complexity \
		-D clippy::perf \
		-D clippy::style \
		-D clippy::suspicious \
		-W clippy::nursery \
		-W clippy::pedantic \
		-A clippy::let-underscore-drop \
		-A clippy::multiple-crate-versions \
		-A clippy::non-ascii-literal \
		-A clippy::too-many-lines \
		-A clippy::single-match-else
	python3 tests/scripts/save_config.py
# Don't parallelise the tests
	-cargo test -- --test-threads=1
	python3 tests/scripts/restore_config.py

install:
	cargo install --force --path .

install-dev:
	cargo install --debug --force --path .
