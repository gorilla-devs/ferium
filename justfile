default: install-dev
set windows-powershell := true

# Install ferium to cargo's binary folder
install:
  cargo install --force --path .

# Install ferium to cargo's binary folder, but with faster compilation (offline, debug, nightly, parallel frontend)
install-dev $RUSTFLAGS="-Z threads=8":
  cargo +nightly install --offline --debug --force --path .

# Delete test artefacts
clean-test:
  rm -rf tests/mods
  rm -rf tests/md_modpack
  rm -rf tests/cf_modpack
  rm -rf tests/configs/running
