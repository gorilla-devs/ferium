# Changelog for Ferium

This changelog is formatted based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [3.2.0] - 06.08.2021

> A quick note about Octorok
> 
> Octorok is pretty buggy right now. AFAIK GitHub does not provide proper definitions for the structs used by their REST API, hence some fields which are supposed to be nullable may not be. _Please_ file an issue the moment you get an error about Octorok JSON deserialisation failing due to missing fields

### Added

- Added support for GitHub Releases
- Added `Octorok`, a Github API abstraction for Ferium
- Made the help page a 'copy' of the README file with suitable formatting
- Added version command for checking the version
- Added repositories to the configuration

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
