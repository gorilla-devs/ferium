# Ferium

[![Codacy Badge](https://api.codacy.com/project/badge/Grade/53ce46e25b19475d83735e7346166849)](https://app.codacy.com/gh/theRookieCoder/ferium?utm_source=github.com&utm_medium=referral&utm_content=theRookieCoder/ferium&utm_campaign=Badge_Grade_Settings)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

Ferium is an open source and easy to use package manager for Minecraft mods on [Modrinth](https://modrinth.com) and [Github Releases](https://github.com/releases). Simply add the mods you use (through the CLI or config file) and in just one command, you can download all the mods and update existing ones.

## Feature to do

I hope to implement all the following features, preferably in the order listed:

- [x] Allow configuration through a CLI
- [x] Add support for downloading from Github releases
- [x] Add support for checking Minecraft version compatibility
- [x] Add support for checking mod loader compatibility
- [x] Command to remove mods or repositories
- [x] Prompt for settings when initialising config file
- [x] Add proper error handling
  - [x] With descriptive error messages
- [ ] Make both mods and repos appear in one list for `ferium remove`
- [ ] Command to change settings (output directory, version, loader)
- [ ] Improve Github repository version checking
- [ ] Make program not redownload when mod is already downloaded and up to date
- [ ] Add `man` page
- [ ] Coloured terminal output
- [ ] ~~Release the Labrinth library as a seperate Crate~~ Migrate to [Femtorinth](https://github.com/phnixir/femtorinth/)
- [ ] Add support for downloading from CurseForge (unlikely)
