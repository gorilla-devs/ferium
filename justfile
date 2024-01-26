default: install-dev
set windows-powershell := true

# Install Ferium to cargo's binary folder
install:
    cargo install --force --path .

# Install Ferium to cargo's binary folder but debug
install-dev:
    cargo install --debug --force --path .

# Delete all build and test artefacts
clean:
    cargo clean
    rm -rf tests/mods
    rm -rf tests/md_modpack
    rm -rf tests/cf_modpack
    rm -rf tests/configs/running
