build-mac:
	cargo clean
	rm -f out/ferium-macos-x64.zip out/ferium-macos-arm.zip
	mkdir -p out

	cargo build --target=x86_64-apple-darwin --release
	cargo build --target=aarch64-apple-darwin --release

	zip -r out/ferium-macos-x64.zip -j target/x86_64-apple-darwin/release/ferium
	zip -r out/ferium-macos-arm.zip -j target/aarch64-apple-darwin/release/ferium

build-win:
	cargo clean
	rm -f out/ferium-windows-gnu.zip
	mkdir -p out

	cargo build --target=x86_64-pc-windows-gnu --release

	zip -r out/ferium-windows-gnu.zip -j target/x86_64-pc-windows-gnu/release/ferium

build-linux:
	cargo clean
	rm -f out/ferium-linux-gnu.zip
	mkdir -p out

	cargo build --target=x86_64-unknown-linux-gnu --release

	mkdir out
	zip -r out/ferium-linux-gnu.zip -j target/x86_64-unknown-linux-gnu/release/ferium

install:
	cargo install --force --path . --root ~

install-dev:
	cargo install --debug --force --path . --root ~ 
