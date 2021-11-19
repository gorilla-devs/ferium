# Changelog for Ferium

## [3.8.0] - 19.11.2021

- Switched to `octocrab` and removed `octorok`
- Lots of code has been refactored and many comments and docs have been edited and improved
- The name of config file fields have been edited so that they more accurately represent the data they hold. **This breaks previous configs** and requires editing field names to fix
- In the config file, repos are now stored as tuples rather than structs
- Added verbose option to the list command. So now the non-verbose list command gives less information, but is faster
- Improved error handling
- Tweaked semver patch remove function so that it works on semvers other than Minecraft's
- Switched from `fancy-regex` to `onig`
- Added integration tests (**check the readme on how to properly run these**) and some unit tests too. These tests run before makefile builds

## [3.7.2] - 06.11.2021

- Switched to `thiserror` for error handling

## [3.7.1] - 06.11.2021

- Ferium now compiles successfully on Linux
- `add_mod()` now adds the mod_id to the config rather than the provided mod ID/slug

## [3.7.0] - 04.11.2021

- Migrated to [Ferinth](https://crates.io/ferinth)
- Fixed a bug where GitHub API requests are using HTTP rather than HTTPS
- Added developers field to `ferium list`
- Tweaked `ferium list` formatting
- Made internet connection check come after clap subcommand parsing so that `ferium help` and `ferium version` can be run without an internet connection
- Tweaked `upgrade_modrinth()` version compatibility checking

## [3.6.1] - 01.11.2021

- Switched to Clap's GitHub repo to get Clap v3
- Added starting directory argument to `pick_folder()`
- Some wording changes in `cli.yaml`
- Some small tweaks in `cli.rs` to support Clap v3
- In `json.rs`, the first time config has been moved to `Config::new()`
- Previously when using a `dialouger` picker, the message was printed using `println!()` then the picker was rendered. Now that message has been moved to the `with_prompt()` modifier
- `configure()` now loops until Quit is selected
- Some other small improvements to `ferium config`	
- Tweaked `list()`'s output

## [3.6.0] - 30.10.2021

- Added a check for mod loader compatibility in `upgrade_modrinth()` and `upgrade_github()`
- Improved the GitHub Releases version and mod loader checking algorithm
- `download_release()` and `download_version()` are now `download_asset()` and `download_version_file()` repectively
- If multiple compatible assets were found, a selector is shown to let the user pick the latest version
- The progress indicators updates now show the name of the asset being downloaded rather than the name of the `Version` or `Release`
- Added function `remove_minor_version()` which is extracted from `get_latest_mc_versions()`

## [3.5.1] - 28.10.2021

- Migrated from `native-dialog` to [`rfd`](https://crates.io/crates/rfd)
- Removed `NativeDialogError` as its no longer required
- Removed `wrappers::print()` and replaced it with `eprint!`
- Removed GitHub Actions workflow
- `FError` now prints error messages in its custom implementation of `std::fmt::Debug`
- Made the main function also return an `FResult<()>` which on error, prints the error message from `FError`'s `Debug` and exits with code `1`

## [3.5.0] - 24.10.2021

### Added

- Build and release workflow
- `config` command

### Changed

- Internet connection timeout
- Improved `remove` command by showing mods and repos at once
- `ferium list` formatting
- `get_config_file` returns `None`, rather than erroring out, after first time setup
- `Select` and `MultiSelect` use the colorful theme
- Switched to Mozilla Public License 2.0
- Functions which change `config` values now don't write to the `config_file`. The main function does so with those functions receiving a `&mut Config` instead
- All the Todo list items have been moved to [a GitHub Project](https://github.com/theRookieCoder/ferium/projects/1) and `README.md`'s todo list section has been removed

### Fixed

- Mod files for Github Releases now use the correct names
- Repositories which do not release anything no longer crash the program
- Creation of output directory before `upgrade`ing

## [3.4.0] - 23.10.2021

- Upgraded to 2021 edition
- Added `make install` to compile and install `ferium`
- Added proper error checking! (no more `unwrap()`s and `panic!()`s, only `?`s)
- Added check for an internet connection
- Improved check for empty config file
- Added `remove` command to remove to remove mods or repositories from config
- Improved checking of releases for `.jar` assets
- Removed `does_exist` for mod versions, use `match get_mod(...)` instead
- Added checking of releases and versions for mc version and mod loader compatibility
- Converted using `format!()`s for path manipulation to using `pathbuf.join()`
- Made `clap` use version from `crate_version` rather than hardcoding it
- Added `FError` and `FResult` for error checking support
- Added first time setup where user selects mod loader, mc version, and mods directory
- Added abstractions for Mojang's launcher_meta API
- Added function to get `n` of the latest versions of Minecraft (using launcher_meta)
- 

## [3.3.0] - 17.08.2021

### Added

- Some metadata in `cargo.toml`
- Improved CLI to use Clap's built in `version` and `help` subcommands

### Changed

- Renamed `funcs.rs` to `calls.rs` in Labrinth and Octorok
- Removed glob imports where possible
- Switched deserialisation of the file to Serde's built in `from_reader`
- The relative flag in `request` has been replace with a `relative_request` function
- Improved file manipulation in `main.rs` and `wrappers.rs` to use `.join()`s instead of `format!()`
- Removed all `match` and `exit()` pairs to improve error handling in the future. _For now_ these have been replaced with `unwrap()`s
- Made `print()` accept `impl Display` to decrease `String` copies

## [3.2.0] - 06.08.2021

### Added

- Support for GitHub Releases
- `Octorok`, a Github API for Rust
- Made the help page a 'copy' of the README file with suitable formatting
- Version command for checking the version
- Repositories to the configuration

### Changed

- Made HTTP calls non-blocking and asynchronous
- Made all HTTP calls reuse a predefined client
- Added more documentation
- Made `make` builds timed

## [3.1.2] - 06.08.2021

### Added

- Added a makefile for building this project
- Added full documentation for Labrinth structs

### Changed

- Updated `cli.yaml`'s documentation to match the help page
- Moved around Labrinth struct definitions to match its documentation

## [3.1.1] - 05.08.2021

### Changed

- Made the `License` struct's `url` field nullable

## [3.1.0] - 05.08.2021

### Added

- This changelog
- Command line interface and corresponding code (under `cli.rs` and `cli.yaml`)
- Help page and command, add mod command, list mods command
- `Users` struct for Labrinth
- JSON file write
- `does_exist()` function for checking if a mod exists
- Error codes (see changed)

### Changed

- Moved utilities to `util` folder
- Made all panics into `println!`s and exits (with descriptive error codes)
- Commented code better
