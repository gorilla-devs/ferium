build-mac:
	cargo clean
	# For macOS on Intel
	cargo build --target=x86_64-apple-darwin --release
	# For macOS on Apple Silicon
	cargo build --target=aarch64-apple-darwin --release

build-win:
	cargo clean
	# For Windows x64
	cargo build --target=x86_64-pc-windows-gnu --release

build-linux:
	cargo clean
	# For Linux x86-64
	cargo build --target=x86_64-unknown-linux-gnu --release

install:
	cargo install --force --path . --root ~

install-dev:
	cargo install --debug --force --path . --root ~ 
