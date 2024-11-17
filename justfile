default: install-dev
set windows-powershell := true

# Install ferium to cargo's binary folder
install:
  cargo install --force --path .

# Install ferium to cargo's binary folder, but with faster compilation (offline & debug)
install-dev:
  cargo install --offline --debug --force --path .

# Delete test artefacts
clean-test:
  rm -rf tests/mods \
    tests/md_modpack \
    tests/cf_modpack \
    tests/configs/running
