build-mac:
# Run clippy
	cargo clippy -- -D clippy::all
# Test with the specific target triple
	python3 save_config.py
	echo "Please remove both mods on the final 'remove' test"
	-cargo test --target=x86_64-apple-darwin -- --test-threads=1
	python3 restore_config.py
# Test with the specific target triple
	python3 save_config.py
	echo "Please remove both mods on the final 'remove' test"
	-cargo test --target=aarch64-apple-darwin -- --test-threads=1
	python3 restore_config.py
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
# Run clippy
	cargo clippy -- -D clippy::all
# Test with the specific target triple
	python3 save_config.py
	echo "Please remove both mods on the final 'remove' test"
	-cargo test --target=x86_64-pc-windows-gnu -- --test-threads=1
	python3 restore_config.py
# Remove previous builds
	rm -f out/ferium-windows-gnu.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-pc-windows-gnu --release
# Zip and move executable to out/
	zip -r out/ferium-windows-gnu.zip -j target/x86_64-pc-windows-gnu/release/ferium.exe

build-linux:
# Run clippy
	cargo clippy -- -D clippy::all
# Test with the specific target triple
	python3 save_config.py
	echo "Please remove both mods on the final 'remove' test"
	-cargo test --target=x86_64-unknown-linux-gnu -- --test-threads=1
	python3 restore_config.py
# Remove previous builds
	rm -f out/ferium-linux-gnu.zip
# Make builds output directory if it doesn't exist
	mkdir -p out
# Build for targets
	cargo build --target=x86_64-unknown-linux-gnu --release
# Zip and move executable to out/
	zip -r out/ferium-linux-gnu.zip -j target/x86_64-unknown-linux-gnu/release/ferium

test:
	python3 save_config.py
	echo "Please remove both mods on the final 'remove' integration test"
	-cargo test -- --test-threads=1
	python3 restore_config.py

install:
	cargo install --force --path . --root ~

install-dev:
	cargo install --debug --force --path . --root ~ 
