# Changelog for Ferium

## `v4.5.2`
### 23.02.2024

- Pad the file sizes when downloading to the right so that the units line up
- Only print the message `Downloading Modpack` when actually doing so
- Properly pad the version-resolution messages based on the largest mod name instead of an arbitrary length
- Many improvements to libium, including;
  - Omission of the active index and profiles/modpacks fields in the config file if they're zero or empty
  - Fix a bug where the directory to which a file was being downloaded would not be created ([#402](https://github.com/gorilla-devs/ferium/issues/402))
  - Faster Modrinth mod adding

## `v4.5.1`
### 07.02.2024

- Add support for the [NeoForge](https://neoforged.net) mod loader ([#392](https://github.com/gorilla-devs/ferium/issues/392))
- Fix coloured output not working in Windows `conhost` ([#148](https://github.com/gorilla-devs/ferium/issues/148))
- Replace the slow, wasteful, and unreliable internet check with an appended warning when the program errors out due to a possible internet connection failure
- Make markdown verbose listing use batch requests too
- Make CurseForge mods use a batch request for verbose listing too
- Properly update the cached names for Modrinth mods again
- Make non-markdown verbose-listing display in alphabetical order ([#315](https://github.com/gorilla-devs/ferium/issues/315))
- Fix a bug where the mod loader in a GitHub Release asset filename wasn't detected properly if it was at the end, i.e. something like `*-<mod-loader>.jar` ([#343](https://github.com/gorilla-devs/ferium/issues/343))

## `v4.5.0`
### 30.01.2024

- Added aliases for many commands and arguments
- The `--version`/`-v` flags only work for the top-level command now (they do not propagate to subcommands)
- Renamed `--dont-check-game-version` and `--dont-check-mod-loader` to `--ignore-game-version` and `--ignore-mod-loader` respectively
  - Also added a short hand `-V` and `-M`
  - The previous flags still work as a hidden alias, so scripts do not have to be edited
- Added `ferium profile info` and `ferium modpack info` subcommands to print information about the current profile/modpack
  - These are aliased to `ferium profile` and `ferium modpack`
- Added `ferium profiles` and `ferium modpacks`, which are aliases to their respective list commands
- Added option to temporarily ignore game version and mod loader checks and force-add the mod anyways ([#142](https://github.com/gorilla-devs/ferium/issues/142))
- Added an argument for providing a profile/modpack to switch to after deleting a profile/modpack ([#390](https://github.com/gorilla-devs/ferium/issues/390))
- Fixed errors not getting caught when adding mods (e.g. `HTTP error 404` instead of `mod does not exist`)
  - Moved a majority of the mod adding code to libium
- Added a header with profile information in `ferium list`
- Made `ferium list` display the source and mod ID first and mod name last to circumvent minor formatting issues
- Added user agent to GitHub API check to make it more reliable
- Added more information and coloured formatting to profile and modpack pickers (when deleting or switching)
- Made the remove picker show the same information as `ferium list` but without colours
- Fixed distribution denied errors not getting caught during mod resolution and causing the entire command to fail instead ([#308](https://github.com/gorilla-devs/ferium/issues/308))

## `v4.4.1`
### 07.08.2023

- Update dependencies
  - Fix [#363](https://github.com/gorilla-devs/ferium/issues/363) by updating ferinth
- Defer GitHub client builder since a [recent regression](https://github.com/XAMPPRocky/octocrab/issues/396) leads to `build()` taking a very long time on macOS
  - Commands like verbose listing, adding github mods, and upgrading will take longer to start up as a result
- Include the new CurseForge API key
- [#310](https://github.com/gorilla-devs/ferium/issues/310): Ping the APIs themselves to check for an internet connection instead of using `online`

## `v4.4.0`
### 24.03.2023

- Updated dependencies
- Improved workflows
- Moved clippy lints to be in-code
  - Removed `lint` just recipe
  - Removed lint flags from `build.yml`
- Fixed [#285](https://github.com/gorilla-devs/ferium/issues/285)
- Refactor `upgrade.rs`
- Fixed [#274](https://github.com/gorilla-devs/ferium/issues/274)
- Added dependency overrides to CurseForge too
- Fixed [#266](https://github.com/gorilla-devs/ferium/issues/266)
- Improved CLI documentation
- Tweaked integration tests
- Made `--threads` global argument configure tokio worker thread count
- Properly batched Modrinth verbose list requests
- Updated local name cache when verbose listing

## `v4.3.4`
### 15.01.2023

- Update dependencies
- Fix [#262](https://github.com/gorilla-devs/ferium/issues/262) by adding additional 'show more' options to the Minecraft version picker

## `v4.3.3`
### 01.01.2023

Update dependencies and removed unnecessary ones

## `v4.3.2`
### 30.12.2022

- Remove more unnecessary `Arc`s
- Tweak progress bar styles

## `v4.3.1`
### 27.12.2022

Switch to `once_cell`

## `v4.3.0`
### 26.12.2022

- Use `JoinSet` instead of a hard-coded loop to slightly speed up parallel performance
- Partially implement [#127](https://github.com/gorilla-devs/ferium/issues/127) by hard coding an override of Fabric API with QFAPI when Quilt is used
- Loosen dependency specification to minor versions only
- Use inline format variables
- Fix [#234](https://github.com/gorilla-devs/ferium/issues/234) by not copying user mods when Quilt is used
- Fix [#230](https://github.com/gorilla-devs/ferium/issues/230) by only copying jar files from the user folder

## `v4.2.2`
### 14.11.2022

Fixed a bug where `Bad file descriptor (os error 9)` is returned when the config file is created.

## `v4.2.1`
### 13.11.2022

- Update dependencies
- Removed unnecessary `Arc`s, `clone()`s, and `async`s
- Set the `download.rs` progress bar length immediately
- Use a common reqwest client for downloads
- Fix [#229](https://github.com/gorilla-devs/ferium/issues/229)
- Fix [#228](https://github.com/gorilla-devs/ferium/issues/228) by updating to libium `1.21.0`
- Use the `inc()` method with progress bars when possible

## `v4.2.0`
### 03.10.2022

- Fix [#184](https://github.com/gorilla-devs/ferium/issues/184), switch all linux builds to use `musl`
- Fix [#134](https://github.com/gorilla-devs/ferium/issues/134), add arm builds for linux
- Make both the GitHub PAT and custom CF API key have a global flag and environment variable

## `v4.1.11`
### 02.10.2022

- Upgrade dependencies and corresponding code
- Replaced unwraps with expects
- Replaced CLI attribute helps/abouts with doc comments
- Remove `gtk` feature and rename `xdg` feature to `gui`
- Fix [#173](https://github.com/gorilla-devs/ferium/issues/173) and [#158](https://github.com/gorilla-devs/ferium/issues/158)
- Implement linter suggestions
- Use batch requests for modrinth verbose listing

## `v4.1.10`
### 25.07.2022

Properly enable file dialogue

## `v4.1.9`
### 25.07.2022

- Fix [#181](https://github.com/gorilla-devs/ferium/issues/181), create the backup directory before copying files over
- Check that there are modpacks before listing them

## `v4.1.8`
### 18.07.2022

Fix [#87](https://github.com/gorilla-devs/ferium/issues/87) by updating to libium `1.19.2`

## `v4.1.7`
### 17.07.2022

- Fix [#172](https://github.com/gorilla-devs/ferium/issues/172)
- Actually fix [#139](https://github.com/gorilla-devs/ferium/issues/139) by moving the subcommand's code to the very top

## `v4.1.6`
### 17.07.2022

- Update dependencies and their respective breaking code
- Fix [#139](https://github.com/gorilla-devs/ferium/issues/139), complete subcommand now runs without reading the config
- Fix [#169](https://github.com/gorilla-devs/ferium/issues/169) by updating to ferinth `2.5.0`
- Fix [#152](https://github.com/gorilla-devs/ferium/issues/152) by catching all errors in optional dependencies and turning them into warnings
- Added proper ueer agent information from the new ferinth update
- Fix [#157](https://github.com/gorilla-devs/ferium/issues/157) added custom CF API key env var

## `v4.1.5`
### 18.06.2022

- Added a `--markdown` flag to the list verbose command to output in markdown format
  - This is useful for modpack mod lists for example
- Updated dependencies

## `v4.1.4`
### 12.06.2022

- Fixed bug where if the slice is shorter than 2, the find dupe functions will panic
- Added an env var to set the config file
- Updated ferinth to `2.3.0`

## `v4.1.3`
### 11.06.2022

- Clean up imports
- Fix [#131](https://github.com/gorilla-devs/ferium/issues/131), clean up `main.rs`'s fetching of the current profile and modpack
- Fix [#130](https://github.com/gorilla-devs/ferium/issues/130)
- Fix [#126](https://github.com/gorilla-devs/ferium/issues/126). Removed `sort` subcommand, all profiles' mods are sorted when writing to config

## `v4.1.2`
### 08.06.2022

- Fix [#113](https://github.com/gorilla-devs/ferium/issues/113), dependencies will no longer be auto updated to minor versions
- Fix [#118](https://github.com/gorilla-devs/ferium/issues/118), the `--output-dir` flag is now optional
- Fix [#111](https://github.com/gorilla-devs/ferium/issues/111)
- Fix [#121](https://github.com/gorilla-devs/ferium/issues/121), the add command just shows a warning for optional mods that are incompatible
- Fix [#120](https://github.com/gorilla-devs/ferium/issues/120), duplicate files are no longer downloaded. Also, a warning so that you remove the duplicate mod

## `v4.1.1`
### 31.05.2022

Fix [#103](https://github.com/gorilla-devs/ferium/issues/103) by updating furse

## `v4.1.0`
### 31.05.2022

- [#65](https://github.com/gorilla-devs/ferium/issues/65) Adding mods and modpacks is now done with one command
- Fixed [#102](https://github.com/gorilla-devs/ferium/issues/102)
- [#74](https://github.com/gorilla-devs/ferium/issues/74) Added `complete` command to generate shell auto completions
- Improved command help messages

## `v4.0.3`
### 30.05.2022

- Improve add commands' output
- Implement Libium's changed distribution denied error handling

## `v4.0.2`
### 24.05.2022

- Change the default feature to GTK (bug)
- The donation suggestion will only pop up if the vector is not empty

## `v4.0.1`
### 22.05.2022

- [#77](https://github.com/gorilla-devs/ferium/issues/77) Implement Libium's new XDG gui backend
- [#91](https://github.com/gorilla-devs/ferium/issues/91) Unhide the `--config-file` flag
- Use Rust 1.61's newly stabilised `std::process::ExitCode`
- Use semaphore in mods upgrade to fix [#89](https://github.com/gorilla-devs/ferium/issues/89)
- Batch request all the files in a CurseForge modpack at once
- Show an error message when a file in a modpack can't be downloaded because of the monopolistic dystopia that has become the largest Minecraft mod distribution website
- Improve third party restricted mods message when downloading a modpack

## `v4.0.0`
### 19.05.2022

Support for Modrinth and CurseForge modpacks!

**_WARNING_**: The config file has had breaking changes.
Add the following to the top of your config file at `~/.config/ferium/config.json` to migrate it:
```json
"active_modpack": 0,
"modpacks": [],
```

- Build workflow now runs when `Cargo.lock` is changed
- Improve `README.md` and add information about modpack subcommands
- Update clippy lints to be stricter
- Added `modpack` subcommand
  - Added integration tests for these too
- Moved `switch` subcommand to `profile switch`
- Added donation hints after adding a Modrinth mod
  - This is to sort of compensate for the fact that no ad revenue is given to mod authors if you use Ferium
- Downloading now shows the file size rather than the number of mods
  - _The progress bar is a liiiieeeeee_ (at first, but it stabilises later)
- Downloads are now stable for 100> mods
- Downloads now directly write to part files then rename the file to its proper name after downloading is finished. Part files will be deleted if found

## `v3.28.7`
### 12.05.2022

- Added the minor version to all dependencies because `cargo install` doesn't use the lockfile
- Update to Libium 1.14.1
- The CurseForge API key is now written in source, no more environment variable! This is allowed as explained in the source code
- Added `--dont-add-dependencies` flag to add commands ([#63](https://github.com/theRookieCoder/ferium/issues/63))
- Tweak progress bar style colours
- Changed the way the progress bar status is updated so that the spinner is smooth
- Added coloured formatting to profile listing too

## `v3.28.6`
### 11.05.2022

- `add-github` now accepts the `theRookieCoder/ferium` format (only one argument)
  - This makes it easier to copy paste the name from the URL
- Fixed a bug when adding dependencies
- Improved the non-verbose list command by showing the source and identifier

## `v3.28.5`
### 11.05.2022

- Added a progress bar that looks very cool when upgrading
  - Not that you can see it for that long anyways :P

## `v3.28.4`
### 11.05.2022

- Added project id to list verbose and some other small tweaks
- Upgrading now deletes old files if moving the file failed (e.g. if it's already in `.old`)
  - This fixes [#60](https://github.com/theRookieCoder/ferium/issues/60)

## `v3.28.3`
### 11.05.2022

- Update to Libium 1.14
  - This fixes [#53](https://github.com/theRookieCoder/ferium/issues/53)
- Update Releases description
- Update the way the GitHub PAT is applied to the API
  - This fixes [#52](https://github.com/theRookieCoder/ferium/issues/52)
- Significantly improved verbose listing to be more colourful, consistent, and informative

## `v3.28.2`
### 09.05.2022

- Update to Libium 1.12
- Improved upgrade code to be faster and more clean
- Immediately fail if rate limit error occured
  - Somewhat fixes [#51](https://github.com/theRookieCoder/ferium/issues/51)
- Show the file size when downloading files

## `v3.28.1`
### 08.05.2022

- Adding github mods now checks tried to get the latest compatible version
- Fixed [#47](https://github.com/theRookieCoder/ferium/issues/47) by using Libium 1.11.4

## `v3.28.0`
### 08.05.2022

Upgrading and verbose listing of mods is now _**SUPER**_ fast compared to before (14-20 times) due to multi threading

- Added multi threading for getting latest mod versions and downloading mods
- Added `--threads` options to limit the maximum number of additional threads
- Used `Arc` in many locations to use the APIs without having to _actually_ clone them
- Added `mutex_ext` to (somewhat unsafely) recover from a poison error and lock a mutex
- If a CurseForge request fails during version determination with a status code, then the request is tried again
  - Requests are sent so fast the CF API gives 500 internal server errors sometimes

## `v3.27.0`
### 07.05.2022

- Added a `.old` directory in the output directory to store 'deleted' mods
- Update the output directory checking code so that the backup is only requested when there are files (because directories will not be deleted)
- Added support for user installed mods
  - These are read from `<output_dir>/user`
- There are now download and install messages when upgrading

## `v3.26.0`
### 05.05.2022

- The `remove` command now uses arguments instead of flags
  - Mod names with spaces have to be given in quotes (`ferium remove "ok zoomer"`) or the spaces should be escaped (`ferium remove ok\ zoomer`)
- Extracted minecraft version picking in `configure` and `create` into `mod.rs`
- Removed checking of Minecraft version when creating profile using arguments
  - Also removed the related `--force_game_version` flag and integration test
- Added an `--import` flag to `profile create`
  - If only `--import` is provided, then the profile to import mods from will be asked
  - If `--import "Profile Name"` is provided, the mods from the `Profile Name` profile will be imported
- Only check internet connection for subcommands which need it
  - Extract internet connection check function
- Created a `check_output_directory()` that uhhh.. checks the output directory
  - If it's not called `mods`, it will output a warning
  - If it contains stuff, it will ask if you want to create a backup
    - So that you don't lose the stuff in there when you upgrade
  - Along with the original relative directory check
- Made sort command convert to lowercase when sorting

## `v3.25.1`

- Updated Libium and Ferinth, to fix 2 issues
- CurseForge mods that have no logo no longer cause errors
- The default mods directory now uses `Application Support` rather than a shortcut to it, `ApplicationSupport` (thanks [douira](https://github.com/douira)!)

## `v3.25.0`
### 04.05.2022

- Added `--dont_check_game_version` and `--dont_check_mod_loader` flags to add commands
  - The check overrides are follow when checking for dependencies
  - These check overrides will be added to the config automatically
- Added Quilt->Fabric backwards compatibility to the add commands' dependency checking

## `v3.24.0`
### 03.05.2022

 - Updated to Libium `1.11`
   - Quilt -> Fabric backwards compatibility is now handled by Ferium rather than Libium
 - Updated the integration tests to use Incendium rather than Terralith because the CF API currently has some problems with Terralith
 - Calls to `libium::upgrade` functions no longer provide a `Profile`, the game version and mod loader to check are given instead
 - Getting the primary file of a Modrinth `Version` has been moved to the conversion function
   - A workaround with vectors is being used to avoid a possible borrow checker bug
 - Ferium will now only rely on Fabric backwards compatibility if it can't find a native Quilt version. This fixes [#30](https://github.com/theRookieCoder/ferium/issues/30)

## `v3.23.0`
### 01.05.2022

- All crates have their minor version specified in `Crates.toml`
- The workflow will now publish to crates.io too
- Update README
- Move upgrading code in `main.rs` to `subcommands::upgrade`
- Added dependency handling to `subcommands::add`
  - Required dependencies will automatically be added
  - Optional dependencies will be added after the user approves

## `v3.22.2`
### 27.04.2022

Make GitHub Release downloads actually work

## `v3.22.1`
### 25.04.2022

- Add a GitHub personal access token option to the root command
- The integration tests will use the `GITHUB_TOKEN` environment variable if it is available (e.g. during actions workflows)

## `v3.22.0`
### 24.04.2022

- Update to Libium 1.9
- Added static variables for a yellow tick and the dialoguer colourful theme
- Extracted the pick mod loader code to `profile/mod.rs`
- Added a yellow tick if the mod was compatible due to backwards compatibility

## [3.21.0] - 20.04.2022

- Removed `no_patch_check` flag for the upgrade command
- There are now overrides for game version and mod loader checks. For now there is no UI, you have to edit the config file manually

## [3.20.1] - 16.04.2022

When picking a file from a version, Ferium will get the primary file rather than the first file

## [3.20.0] - 16.04.2022

- Added a `Downloadable` struct that represents (and be converted from) a mod file from Modrinth, GitHub Releases, or CurseForge
- There is now also a constant for the cross too ("×")
- Big changes to upgrading:
  - Does not empty the output directory
  - Checks if the latest compatible version is already downloaded, if so it does not download it again
  - If there are files that are not the latest compatible version of a mod, then they are deleted. So _effectively_ the output directory is emptied

## [3.19.1] - 03.04.2022

- Added a sort command that sorts mods in alphabetical order
- Edited the `mod.rs` files to make accessing functions better
- Updated some user facing output such as command help pages, and errors
- Reverted to the old output style in the add command

## [3.19.0] - 02.04.2022

> WARNING!
> 
> The config format has changed, your previous (pre-3.19) configs will not work!
> Use [Frigate](https://github.com/theRookieCoder/frigate) to update your old config to the new format

- Updated to Libium 1.6
- Improved the upgrading code in `main.rs`, taking advantage of the improved config format
- Updated `remove.rs` to support the new libium version
- Updated the test configs to the new format

## [3.18.2] - 31.03.2022

- Improved readme
- Made commands `ferium` and `ferium profile` show the help page when not given any subcommands

## [3.18.1] - 30.03.2022

Update to libium `1.5.1` to remove OpenSSL

## [3.18.0] - 30.03.2022

- Removed `upgrade.rs` and used `libium::upgrade`
- Made errors print out red and bold
- Made ticks green using a const
- Significantly improved the output during upgrades

## [3.17.0] - 28.03.2022

Removed error handling

- Removed `error.rs`
- Replaced `thiserror` with `anyhow`
- `return Err(Error::Quit("Error message))` has been replaced with `bail!("Error message)` from anyhow
- Made upgrade command return an erraneous exit code if downloading failed

## [3.16.0] - 28.03.2022

- Moved `add.rs` to libium
- Small edits to accomodate changes in libium 1.4
- Massively simplified error handling, planning to replace with `anyhow` soon as it useless

## [3.15.3] - 26.03.2022

Small tweaks to accomodate the changes in `ferinth` and `libium`

## [3.15.2] - 26.03.2022

- Added `gui` feature. This changes the file dialogue from a text input to a gui file dialogue
- Tweaked formatting of profile list command

## [3.15.1] - 25.03.2022

- Added hidden `config-file` argument
- Massively improved integration tests. They are now independant from each other and can run in parallel

## [3.15.0] - 24.03.2022

Switched from make to [just](https://github.com/casey/just)

- Added back the verbose flag to the list command. The verbose version is the same as before, and the non-verbose version lists out all mod names only
- Small edits to the add command to support the new config format
- The list command now uses a more concise method to display lists of developers/categories (hence the new dependency `itertools`)
- Many commands have been converted to work on one project at a time so there are less loops in functions and more in main.rs
- The remove command has been significantly improved and is now much faster due to the new name storing format
- The remove command now also requires argument provided names to be exactly the same
- Simplified delete profile command

## [3.14.0] - 06.03.2022

- All print statements in add and upgrade functions have been removed, this means
- Errors now only have the error and no formatting
- Add GitHub repo function now accepts a `RepoHandler` rather than the repo name
- All add functions now return the added item
- Adding Modrinth mods now checks if the project is a mod
- Writing to mod files has been extracted to `write_mod_file()`
- All upgrade commands only download a single mod
- Downloading GitHub mods no longer picks from multiple mods
- The output directory is now emptied before mods are downloaded

## [3.13.0] - 02.03.2022

- Moved all the subcommand functions in `main.rs` to seperate files
- Updated the settings in `cli.rs`
Functionality should not have been changed

## [3.12.1] - 11.02.2022

Refactor 'mod' to 'project'

## [3.12.0] - 06.02.2022

Migrated to [Libium](https://crates.io/crates/libium)!

- Removed unneeded dependencies
- Removed the following which were replaced by libium:
  - `json.rs`
  - `launchermeta.rs`
  - `wrappers.rs`
  - `src/util`
- Moved `util/cli.rs` and `util/ferium_error.rs` to `src/cli.rs` and `src/error.rs`
- Changed errors to the more typical style, `Error` and  `Result` in a `src/error.rs`
- Removed `InvalidDeviceError`, libium just panics instead as it should've never been compiled on an incompatible platform
- Move the `Profile::create_ui()` function to the `main.rs` file
- Upgrade to Ferinth v2
- Add `clean` target to makefile
- Add `URLParseError` from Ferinth
- Switch to an asynchronous std using tokio
- Removed verbose flag for list
- Simplify switch profile
- Use libium from crates.io

## [3.11.1] - 28.01.2022

Windows MSVC support!

- The makefile target `build-win` now builds for Windows MSVC
- The makefile target `build-linux` now builds for GNU Linux and GNU Windows
- The makefile targets `install` and `install-dev` now install to Cargo's `bin` directory
- `install-dev` is now the default goal in the makefile
- `save_config.py` and `restore_config.py` have been updated to use pathlib so that paths are cross platform and no long *NIX paths
- Added handing for Ferinth's new error
- The integration tests also don't use hardcoded *NIX paths anymore
- `check_empty_config()` checks for curse_project now, this was a bug fixed by [SolidTux](https://github.com/SolidTux) in [PR #2](https://github.com/theRookieCoder/ferium/pull/2)
- Move Python scripts for testing to `tests/scripts/`

## [3.11.0] - 23.01.2022

### CURSEFORGE SUPPORT!

- Renamed `mod_ids` and `repos` to `modrinth_mods` and `github_repos` in `Profile`
- Added `curse_projects` to `Profile`
- Added `add-curseforge` command
- An API key is required to compile the application. Without this, the program will not compile. You can a provide blank value for this and the program will compile but anything using the CurseForge API will not work
- Upgrading mods now uses the downloaded file's actual filename rather than the mod name
- The remove command no long tries to remove the mod file (to be reintroduced later)
- Added 2 flags to the upgrade command
  - `no_picker` When upgrading GitHub mods, if multiple assets are compatible a picker is normally shown. This option disables this and uses the first one
  - `no_patch_check` Normally, upgrading curse and modrinth mods will check for the full game version, but sometimes mods works between patch versions, so this option skips checking for the patch version. This doesn't affect upgrading GitHub mods
- Made a test for the upgrade command. Will have to implement checking of the mods downloaded later
- Some integration tests have been edited to test curse mods too
- Upgrading GitHub mods now checks the releases' names for the game version too

## [3.10.1] - 17.01.2022
> Unreleased

- Upgrading Modrinth mods now correctly check the mod loader by converting the mod loader name to lowercase 

## [3.10.0] - 16.01.2022

HUGE UPDATE

### Project and Testing
- Upgraded to Clap 3.0
- Removed unit tests
- Added every single sub(sub)command to the integration tests
- The integration tests are now fully automatic because we can now pass options without the interactive UI

### Main
- Getting the config file no longer exits the program early when creating a new config file
- A more helpful error message for when decoding the config file fails
- The `profile create` subcommand now runs seperately before the current profile is read
- Multiple new command have been created such as `list_profiles()`, and `delete()`
- Many commands have been revamped to allow the arguments to be passed through the CLI rather than through a UI
- The `profile configure` command's UI now has an option to change the profile's name

### Arg Parsing
- Removed `cli.yaml` and switched from the deprecated yaml parsing to #[derive] based arg parsing
- Renamed `add`, `add-repo`, and `config` commands to `add-modrinth`, `add-github`, and `configure`
- The `create` and `config` commands are now under a new subcommand `profile`
- `profile delete` and `profile list` subsubcommands have been added
- The following commands have had options added to them so now using the interactive UI is optional. This also allows for fully automatic tests
  - Remove
  - Switch
  - Configure
  - Create
  - Delete

### Error handling
- There are now 2 errors for quitting
  - The `Quit` error stores its error message as a tuple `&'static str` so that raw strings can be used
  - The `QuitFormatted` error stores its error message as a tuple `String` so that `format!()` can be used for more informative error messages

### Configuration (`json.rs`)
- The `mod_loader` field in `Profile` is now an enum
- The `Profile::new()` is now `Profile::create_ui()`
- `create_ui()` now checks that the name provided does not already exist, if so it will ask for a new name
- Getting the path to the config file has been extracted to `get_config_file_path()`
- If `get_config_file()` does not find a config file, it now just creates an empty config, writes to it, and continues to return the config file

## [3.9.0] - 23.12.2021

Merry Christmas and a Happy New Year!

- The previous `Config` is now a `Profile`. The `Profile` has a new field, `name`
- The new `Config` contains a list of profiles and the index of the active profile
- Added a `create` command to create a new profile
- Added a `switch` command to switch between profiles
- A remove command for profiles will be added in the next update
- Switch back to Clap v2 because I'm rewriting the command system in the next update
- Edited makefile to run clippy in `make test` and builds
- Many code changes based on Clippy suggestions

## [3.8.1] - 29.11.2021

- Added `rustfmt.toml` and formatted code according to that
- Upgraded dependencies
- Some code has been modified to support the many more nullable fields in Octocrab (WHY ARE ALL OF THEM `Option`S!?)

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
