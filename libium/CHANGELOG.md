# Changelog for Libium

## `1.32.0`
###

Added filters

- Structs are defined in `config::filter`
- 6 types: mod loader (prefer and any), game version, minor game version, release channel, and filename regex
- Removed game version and mod loader from `Profile`
- Removed `check_mod_loader` and `check_game_version` from `Mod`
- Added `filters` to `Profile` and `Mod`, added `override_filters` option to `Mod`
- Added `pin` to `Mod`
- Rewrote `upgrade::check` to use filters instead
- The method `Filter::filter` will return indices of the matching files
- Mod resolution will now fail with detailed error messages, including if any of the filters produced an empty set, or if intersecting the filtered sets failed
- Added `release_channel` to `DownloadFile`

## `1.31.0`
### Unreleased

- Switched to `std::sync:LazyLock` and removed the `once_cell` dependency
- Added `MODRINTH_API`, `GITHUB_API`, and `CURSEFORGE_API` as lazily initialised global variables so that they don't need to be passed around everywhere
  - Removed `APIs`
- Changed `Downloadable` into `DownloadFile` and added game version and loader data so that it can be used to perform platform agnostic filtering
- Made functions in `upgrade::check` platform agnostic
- Added `ModIdentifier::fetch_version_likes` method to fetch platform agnostic `DownloadFiles` from its platform dependant enum variants
  - Replaces the `get_compatible_downloadable` function in `upgrade::mod_downloadable`

## `1.30.0`
### 09.08.2024

- [`gorilla-devs/ferium#422`](https://github.com/gorilla-devs/ferium/issues/422): Fix a crash when identical files are scanned
- Replace `tokio` IO with `std::io`, and `async_zip` with `zip`
- Use `zip-extensions` for compressing and extracting to directories
  - Replace `modpack::extract_zip` `modpack::compress_dir` with re-exports of `zip_extract` and `zip_create_from_directory` from `zip_extensions`
- Make many functions not `async`
- Downloading modpacks from `upgrade::modpack_downloadable` no longer returns the file, it returns the path instead

## `1.29.0`
### 11.06.2024

- Made argument and return type in `add::parse_id()` owned
- Made `add()` take `ModIdentifier`s instead of `String`s, so the function itself doesn't parse IDs
- Remove duplicate curseforge and modrinth IDs in `add()`
- Added `scan` module
- `scan()` reads the files in the provided directory, sends their hashes to modrinth and curseforge, and return the project/mod IDs provided

## `1.28.0`
### 10.06.2024

- Add `APIs` struct to store and pass ferinth, furse, and octocrab clients together
- Make `ModIdentifierRef::GitHubRepository` have the references inside the tuple
- `ModIdentifierRef` is now `Copy`
- Replace many instances of `&str` with `impl AsRef<str>`
- Change `upgrade::check::github()` to accept a list of asset names instead
  - This is so that both REST and GraphQL responses can be used
- Improved error messages for custom error types

#### Completely reworked project adding

- Simplify error handling since custom catching of "not found" errors is no longer needed
- Added a function to parse a string into either a curseforge, github, or modrinth identifier
- Required information about projects is now sent batched to the relevant APIs
- GitHub batched queries use GraphQL
- `github()`, `curseforge()`, and `modrinth()` do not perform any network requests, they solely use the data provided in their arguments
- All of these functions now perform compatibility checks by themselves, again without any additional network requests

## `1.27.0`
### 21.05.24

- Update dependencies
- Replace references with `AsRef` in as many places as possible
- Replace functions generics with direct `impl`s as much as possible
- Added `add_multiple` and `add_single` functions to `add` from ferium to facilitate adding of multiple mods

## `1.26.2`
### 23.02.2024

Add Fabric backwards compatibility for Quilt when adding Modrinth mods.

## `1.26.1`
### 22.02.2024

Fix a bug where the directory to which a file was being downloaded would not be created.

## `1.26.0`
### 22.02.2024

- Replace `Option<bool>` with `bool` for whether or not to check game version or mod loader
- Added `ModIdentifierRef` and `ModIdentifier.as_ref()` for comparing mod identifiers without cloning
- Replaced many procedural loops with functional alternatives
- Added `Profile.get_version(check_game_version)` and `Profile.get_loader(check_mod_loader)` to replace this common pattern:
    ```rs
    if check_(game_version | mod_loader) {
        Some(&profile.(game_version | mod_loader))
    } else {
        None
    }
    ```
- Use the `game_versions` and `loaders` specified in the Modrinth `Project` struct instead of using version resolution
- Only use `.to_string()` instead of `.into<String>()` when converting a value to a `String`
- Replace `config::file_path()` with `DEFAULT_CONFIG_PATH` which uses `Lazy`
- Extract the file opening to `open_config_file()`
- Move `config::read_file()` to `read_wrapper()` since it is not specific to the `config` module
- Derive and use `Default` for the `config::Config` when creating an empty config
- Skip serialising the active index and profiles/modpacks vectors if they're zero or empty
- Remove the `Option` in `check_game_version` and `check_mod_loader` fields for `Mod`
- Replace `TryFrom<&str>` with `FromStr` for `ModLoader`
- Derive `Copy` for `ModLoader`
- Determine `get_minecraft_dir()` at compile-time
- Set the UNIX permissions when compressing a directory
- Replace `curseforge::read_manifest_file()` and `modrinth::read_metadata_file()` with `read_file_from_zip()`
- Refactor `upgrade::check` to make it more readable
- Remove the subdirectory classification done when converting to a `Downloadable`, modpack installers can do this manually

## `1.25.0`
### 07.02.2024

Support for [NeoForge](https://neoforged.net)

## `1.24.2`
### 05.02.2024

- Fix [#343](https://github.com/gorilla-devs/ferium/issues/343); When checking github assets, check that the name _ends with_ `.jar`, and strip it before splitting the string
- Tweak the distribution denied error message

## `1.24.1`
### 30.01.2024

Fix compilation on linux

## `1.24.0`
### 28.01.2024

- In `add.rs`, add mods to the profile and return only the mod name
- Add option to override compatibility checks when adding
- Update `async_zip` to the latest version `0.0.16`

## `1.23.0`
### 23.03.2023

- Switch to `async_zip`
- Add `name` argument to `pick_folder()`
- Move `get_minecraft_dir()` to root folder, remove `misc` module and `get_major_mc_versions()`
- Reading manifest or metadata files now returns an optional result
- Removed the rather redundant `deser_manifest()` and `deser_metadata()` functions
- Add a recursive `compress_dir()` function
- Tweak Modrinth modpack structs to use ferinth's types
- Tweak `Downloadable`'s file length field's type
- Wrap `Downloadable::download()`'s opened file in a `BufWriter`
- Only update the progress bar after the write is finished
- Remove `mutex_ext` and `force_lock()`

## `1.22.1`
### 01.01.2023

- Only use required features for `zip`
- Switch to `once_cell` and remove `lazy_static`

## `1.22.0`
### 23.12.2022

Loosen dependency specification and remove unnecessary `bytes` dependency

## `1.21.1`
### 13.11.2022

Fixed a bug where the file returned from `config::get_file()` is not readable if it's newly created

## `1.21.0`
### 13.11.2022

- Update dependencies, remove `urlencoding` and `size`
- Remove unnecessary `Arc`s
- Use the website URL to determine that a project is a a mod/modpack on CF
- Simplify `config` module methods
- Remove redundant doc-comments
- File picker now uses sync dialogue on all platforms
- Edit `file_picker.rs` to use the updated feature flags, fixes [gorilla-devs/ferium#228](https://github.com/gorilla-devs/ferium/issues/228)
- The file picker function will now resolve `~` and `.` to the home and cwd respectively
- Added the android PojavLauncher to the default minecraft directory function
- Change the function signature of `check` functions
- Change `Downloadable`'s `size` field into `length`, remove the `Option`, and make it a number
- Remove the `total` closure in `Downloadable::download()`
- Remove `Downloadable::from_file_id()`
- Edit functions in `mod_downloadable.rs` to match those of `check.rs`

## `1.20.0`
### 03.09.2022

- Update dependencies
- Clean up imports in `add.rs`
- Switch to only XDG backend for `rfd`
- `add::modrinth()` and `add::curseforge()` now directly accept the project struct

## `1.19.2`
### 18.07.2022

Fix a bug where the file is not rewound after being written to

Fixes [gorilla-devs/ferium#87](https://github.com/gorilla-devs/ferium/issues/87)

## `1.19.1`
### 17.07.2022

Update dependencies

## `1.19.0`
### 24.06.2022

- Update dependencies
- Make `Downloadable` use `url::Url`

## `1.18.2`
### 12.06.2022

Update ferinth minor version

## `1.18.1`
### 07.06.2022

- Update dependencies
- [gorilla-devs/ferium#113](https://github.com/gorilla-devs/ferium/issues/113) Make dependencies use `~` so that only minor versions are auto updated
- Many small clippy lint fixes

## `1.18.0`
### 30.05.2022

- Improve error messages
- Add functions no longer add the mod to the config
- Modpack manifests will now accept unknown fields
- `DistributionDeniedError` now has mod_id and file_id fields

## `1.16.0`
### 18.05.2022

Implemented CurseForge's third party distribution restrictions

## `1.15.5`
### 18.05.2022

`modpack::add` no longer adds the project to the config

## `1.15.4`
### 17.05.2022

- Add `install_overrides` field to `Modpack` in config
- Change `get_curseforge_manifest` and `get_modrinth_manifest` to `download_curseforge_modpack` and `download_modrinth_modpack` respectively

## `1.15.3`
### 16.05.2022

- Added Modrinth modpacks
- Modpack add commands only return the project struct now
- Change `Downloadable::filename` to `output` which will include the path from the instance directory
- Added `Downloadable::size` for the file size

## `1.15.2`
### 16.05.2022

- `Downloadable::download()` now directly downloads to the output file as a `.part`, it will rename it back to the actual filename after it finishes downloading
- The `progress` closure is now a `total` and `update` closure
- `Downloadable::from_ids()` now properly decodes percent characters (e.g. `%20` -> ` `)

## `1.15.1`
### 15.05.2022

- Update to Furse `1.1.2`
- Add `from_ids` to create a downloadable from a curseforge project and file id

## `1.15.0`
### 14.05.2022

- Added minor versions to all dependencies
- Moved `check` and `upgrade` to `upgrade::check` and `upgrade::mod_downloadable`
- Moved the `Downloadable` to `upgrade`, it also has a new `download()` function
- Added modpacks to the config
- Added `modpack` with a curseforge modpack and a function to add that to the config

## `1.14.1`
### 12.05.2022

- Changed `misc::get_mods_dir()` to `misc::get_minecraft_dir()`, the new function only returns the default Minecraft instance directory
- Added `config::read_file()` and `config::deserialise()`
- The add commands now return the latest compatible _ of the mod
  - Added `Error::Incompatible` to go along with this
- The curseforge add command checks if the project is a mod using the same method as the github add command

## `1.14.0`
### 11.05.2022

Revert back to octocrab

## `1.13.0`
### 10.05.2022

- Move from octocrab to [octorust](https://crates.io/crates/octorust)
  - This fixes [#52](https://github.com/theRookieCoder/ferium/issues/52)
  - (I later realise that even though it does, octocrab was fine)
- Many GitHub related functions have had their signatures changed
- The `upgrade` functions have been slightly updated
- Removed unnecessary `async`s
- Replaced many `Vec<_>`s with `&[_]`
- The add functions now check if mods have the same name too
  - This fixes [#53](https://github.com/theRookieCoder/ferium/issues/53)

## `1.12.0`
### 09.05.2022

- Rename the `upgrade` module to `check`
- Changes in `check`
  - Removed error
  - `write_mod_file()` now takes an output directory rather than a whole file
  - The functions now take a vector of items to search and return a reference to the latest compatible one using an `Option`
  - The modrinth function now return the primary version file along side the version
- Create a new upgrade module which actually does upgrading stuff
  - Functions to get the latest compatible 'item' for each mod source. These functions also implement the Quilt->Fabric backwards compatibility
  - A function to use the previously mentioned functions from a mod identifier to return a downloadable

## `1.11.4`
### 08.05.2022

- Do not check the release name when checking the game version for github releases
  - This fixes Ferium [#47](https://github.com/theRookieCoder/ferium/issues/47)

## `1.11.3`
### 05.05.2022

- Added `prompt` to file pickers
- Used the `default` provided to the no-gui pick folder

## `1.11.2`
### 05.05.2022

Change macOS default mods directory from using the `ApplicationSupport` shortcut to the actual `Application Support` directory

## `1.11.1`
### 04.05.2022

- Updated to Ferinth `2.2`
- Add commands now accept `should_check_game_version` and `should_check_mod_loader`
- They also use this when adding the mod to the config

## `1.11.0`
### 03.05.2022

- Replace the `for` loop in `check_mod_loader()` with an iterator call
- The upgrade functions no longer deal with Quilt -> Fabric backwards compatibility
- Upgrade functions (again) return only the compatible asset they found
- Upgrade functions no longer take a `profile`, they check for compatibility with the `game_version_to_check` and `mod_loader_to_check` provided

## `1.10.0`
### 01.05.2022

- Added minor versions to `Cargo.toml`
- Update to Furse `1.1`
  - Implemented new error type
- Simplified checking if a project had already been added
- `upgrade::github()` now checks that the asset isn't a sources jar

## [1.9.0] - 24.04.2022

- Added Quilt to `ModLoader`
- Added `check_mod_loader()` to check mod loader compatibility
- The upgrade functions now return additional info, whether the mod was deemed compatible through backwards compatibility (e.g. Fabric mod on Quilt)
- Generally improved code in `upgrade`

## [1.8.0] - 20.04.2022

- Added a `check_mod_loader` and `check_game_version` flag to each mod
- They are `None` by default
- If they are `Some(false)` then the corresponding checks are skipped in `upgrade.rs`
- Removed `no_patch_check`, `remove_semver_patch()`, `SemVerError`, and the `semver` dependency

## [1.7.0] - 15.04.2022

- Remove `config` from function names in config module
- Upgrade functions no longer download and write the mod file
- `write_mod_file()`  is now public

## [1.6.0] - 02.04.2022

Update the `config` struct format

## [1.5.0] - 29.03.2022

- Moved `upgrade.rs` from ferium to libium
  - Added improved custom error handling
  - Improved doc comments
  - Made functions return the file/version/asset downloaded (similar to `add.rs`)
  - Changed some variable names

## [1.4.0] - 28.03.2022

- Moved `add.rs` from ferium to libium
  - Added improved custom error handling
- Extracted file dialogues to `file_picker.rs`
