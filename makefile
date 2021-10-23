build-mac:
	cargo clean
	# For macOS aarch64
	cargo build --target=aarch64-apple-darwin --release
	# For macOS x86-64
	cargo build --target=x86_64-apple-darwin --release

build-win:
	cargo clean
	# For Windows x86-64
	cargo build --target=x86_64-pc-windows-msvc --release
	# For Windows on arm
	cargo build --target=aarch64-pc-windows-msvc --release

build-linux:
	cargo clean
	# For Linux x86-64
	cargo build --target=x86_64-unknown-linux-gnu --release
	# For Linux arm
	cargo build --target=aarch64-unknown-linux-gnu --release

install:
	cargo install -v --force --path . --root ~
