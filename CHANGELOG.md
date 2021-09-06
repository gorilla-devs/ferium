# Changelog for Ferium

This changelog is formatted based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

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
