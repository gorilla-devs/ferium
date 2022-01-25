.SILENT: test
.DEFAULT_GOAL := install-dev

build-mac:
	make test
# Remove previous builds
	rm -f out/ferium-macos-x64.zip out/ferium-macos-arm.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-apple-darwin --release
	cargo build --target=aarch64-apple-darwin --release
# Zip and move executable to out/
	zip -r out/ferium-macos-x64.zip -j target/x86_64-apple-darwin/release/ferium
	zip -r out/ferium-macos-arm.zip -j target/aarch64-apple-darwin/release/ferium

build-win:
	make test
# Remove previous builds
	rm -f out/ferium-windows-gnu.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-pc-windows-gnu --release
# Zip and move executable to out/
	zip -r out/ferium-windows-gnu.zip -j target/x86_64-pc-windows-gnu/release/ferium.exe

build-linux:
	make test
# Remove previous builds
	rm -f out/ferium-linux-gnu.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-unknown-linux-gnu --release
# Zip and move executable to out/
	zip -r out/ferium-linux-gnu.zip -j target/x86_64-unknown-linux-gnu/release/ferium

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
	python3 save_config.py
# Don't parallelise the tests
	-cargo test -- --test-threads=1
	python3 restore_config.py

install:
	cargo install --force --path . --root ~

install-dev:
	cargo install --debug --force --path . --root ~ 
