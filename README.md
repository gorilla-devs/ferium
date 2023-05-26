# Ferium

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)
[![licence badge](https://img.shields.io/github/license/gorilla-devs/ferium)](https://github.com/gorilla-devs/ferium/blob/main/LICENSE.txt)
[![build.yml](https://github.com/gorilla-devs/ferium/actions/workflows/build.yml/badge.svg)](https://github.com/gorilla-devs/ferium/actions/workflows/build.yml)

> Check out ferium's sister projects [ferinth](https://github.com/gorilla-devs/ferinth) and [furse](https://github.com/gorilla-devs/furse)  
> They are Rust wrappers for the official Modrinth and CurseForge APIs respectively

Ferium is a fast and feature rich CLI program for downloading and updating Minecraft mods from [Modrinth](https://modrinth.com/mods), [CurseForge](https://curseforge.com/minecraft/mc-mods), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases), and modpacks from [Modrinth](https://modrinth.com/modpacks) and [CurseForge](https://curseforge.com/minecraft/modpacks).
Simply specify the mods you use, and in just one command you can download the latest compatible version of all the mods you configured.

## Features

- Use the CLI to easily automate your modding experience
- Download mods from multiple sources, namely [Modrinth](https://modrinth.com/mods), [CurseForge](https://curseforge.com/minecraft/mc-mods), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- Download modpacks from multiple sources, namely [Modrinth](https://modrinth.com/modpacks) and [CurseForge](https://curseforge.com/minecraft/modpacks)
- <details>
    <summary>Pleasing and beautiful UI</summary>

    - Listing mods
      ![Listing Mods](media/list.png)
    - Listing mods verbosely
      ![Listing Mods Verbosely](media/list%20verbose.png)
    - Upgrading mods/modpacks
      ![Upgrading Mods/Modpacks](media/upgrade.png)
  </details>

- <details>
    <summary>It's super fast due to multithreading for network intensive tasks</summary>

    Your results may vary depending on your internet connection.

    It downloads my modpack [Kupfur](https://github.com/theRookieCoder/Kupfur) with 79 mods in 15 seconds:

    https://user-images.githubusercontent.com/60034030/212559027-2df10657-82a3-407c-875d-9981628bbfc2.mp4

    It downloads [MMTP](https://www.curseforge.com/minecraft/modpacks/mats-mega-tech-pack), as very large modpack with around 400 mods, in just under a minute:

    https://user-images.githubusercontent.com/60034030/201951498-62d1e6d9-8edb-4399-b02c-f2562ae566e3.mp4
  </details>

- Upgrade all your mods to the latest compatible version in one command, `ferium upgrade`
  - Ferium checks that the version being downloaded is the latest one compatible with the configured mod loader and Minecraft version
- Download and install the latest version of your modpack in one command, `ferium modpack upgrade`
- Create multiple profiles and configure different mod loaders, Minecraft versions, output directories, and mods for each

## Installation

Ferium builds from GitHub Releases do not require any external dependencies at runtime.  
If you compile from source on Linux, using GCC to build will result in binaries that require GCC to be available at runtime.  
On Linux, the regular version requires some sort of desktop environment that offers an XDG Desktop Portal to show folder pickers.
The `nogui` versions do not need this as they won't have a GUI folder picker, making these variants suitable for server use.

### Packages

[Coming to more package managers soon](https://github.com/gorilla-devs/ferium/discussions/292)

#### [Arch User Repository](https://aur.archlinux.org) for _Arch Linux_

[![AUR](https://repology.org/badge/version-for-repo/aur/ferium.svg)](https://aur.archlinux.org/packages?K=ferium)

> **Warning**  
> From-source builds will install the Rust toolchain and GCC

| Installation method                             | GUI file dialogue                                                       | No GUI                                                      |
| ----------------------------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------- |
| Install pre-built binaries from GitHub Releases | **[ferium-gui-bin](https://aur.archlinux.org/packages/ferium-gui-bin)** | [ferium-bin](https://aur.archlinux.org/packages/ferium-bin) |
| Build from source at the latest tag             | [ferium-gui](https://aur.archlinux.org/packages/ferium-gui)             | [ferium](https://aur.archlinux.org/packages/ferium)         |
| Build from source using the latest commit       | [ferium-gui-git](https://aur.archlinux.org/packages/ferium-gui-git)     | [ferium-git](https://aur.archlinux.org/packages/ferium-git) |

#### [Homebrew](https://brew.sh) for _macOS_ or _Linux_
[![Homebrew](https://repology.org/badge/version-for-repo/homebrew/ferium.svg)](https://formulae.brew.sh/formula/ferium)
```bash
brew install ferium
```

#### [Scoop](https://scoop.sh) for _Windows_
[![Scoop](https://repology.org/badge/version-for-repo/scoop/ferium.svg)](https://scoop.sh/#/apps?q=ferium&id=d17eaa5fe92af6d5eddb853f06bf27d162cadbba)
```bash
scoop bucket add games
scoop install ferium
```

#### [Pacstall](https://pacstall.dev) for _Ubuntu_
[![Pacstall](https://repology.org/badge/version-for-repo/pacstall/ferium.svg)](https://pacstall.dev/packages/ferium-bin)
```bash
pacstall -I ferium-bin
```

#### [Nixpkgs](https://nixos.wiki/wiki/Nixpkgs) for _NixOS_ or _Linux_
[![Nixpkgs unstable](https://repology.org/badge/version-for-repo/nix_unstable/ferium.svg)](https://search.nixos.org/packages?show=ferium&channel=unstable)  
**Note** See package page for installation instructions

#### [crates.io](https://crates.io) for the _Rust toolchain_
[![crates.io](https://repology.org/badge/version-for-repo/crates_io/rust:ferium.svg)](https://crates.io/crates/ferium)
```bash
cargo install ferium
```
> **Warning**  
> Remember to use an add-on like [cargo-update](https://crates.io/crates/cargo-update) to keep ferium updated to the latest version!

#### GitHub Releases
[![GitHub Releases](https://img.shields.io/github/v/release/gorilla-devs/ferium?color=bright-green&label=github%20releases)](https://github.com/gorilla-devs/ferium/releases)
> **Warning**  
> You will have to manually download and install every time there is a new update

1. Download the asset suitable for your operating system from the [latest release](https://github.com/gorilla-devs/ferium/releases/latest)
2. Unzip the file and move it to a folder in your path, e.g. `~/bin`
3. Remember to check the releases page for any updates!

## Overview / Help Page

> **Note**  
> A lot of ferium's backend is in a separate project [libium](https://github.com/theRookieCoder/libium).  
> It deals with things such as the config, adding mod(pack)s, upgrading, file pickers, etc.

### Config File Location

Ferium stores profile and modpack information in its config file. By default, this is located at `~/.config/ferium/config.json`.  
You can change this in 2 ways, setting the `FERIUM_CONFIG_FILE` environment variable, or setting the `--config-file` global flag.
The flag always takes precedence.

You can also set a custom CurseForge API key or GitHub personal access token using the `CURSEFORGE_API_KEY` and `GITHUB_TOKEN` environment variables or the `--curseforge_api_key` and `--github-token` global flags respectively.
Again, the flags take precedence.

### First Startup

You can either have your own set of mods in what is called a 'profile', or install a modpack.

- Create a new profile by running `ferium profile create` and entering the details for your profile
  - Then, [add your mods](#adding-mods) using `ferium add`
  - Finally, download your mods using `ferium upgrade`
- [Add a modpack](#adding-modpacks) by running `ferium modpack add <project_id>`
  - After which, run `ferium modpack upgrade` to download and install the modpack

### Adding Mods

- Modrinth
  - `ferium add project_id`
  - Where `project_id` is the slug or project ID of the mod
    - For example, [Sodium](https://modrinth.com/mod/sodium) has the slug `sodium` and project ID `AANobbMI`
    - You can find the slug in the website URL (`modrinth.com/mod/<slug>`), and the project ID at the bottom of the left sidebar under 'Technical information'
  - So to add [Sodium](https://modrinth.com/mod/sodium), you can run `ferium add sodium` or `ferium add AANobbMI`
- CurseForge
  - `ferium add project_id`
  - Where `project_id` is the project ID of the mod
    - For example, [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith) has the project id `513688`
    - You can find the project id at the top of the right sidebar under 'About Project'
  - So to add [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith), you should run `ferium add 513688`
- GitHub
  - `ferium add owner/name`
  - Where `owner` is the username of the owner of the repository and `name` is the name of the repository (both case-insensitive)
    - For example [Sodium's repository](https://github.com/CaffeineMC/sodium-fabric) has the id `CaffeineMC/sodium-fabric`
    - You can find these at the top left part of the repository's page as a big 'owner / name'
  - So to add [Sodium](https://github.com/CaffeineMC/sodium-fabric), you should run `ferium add CaffeineMC/sodium-fabric` (again, case-insensitive)
  - **Note**  
    The GitHub repository has to release JAR files in their Releases for ferium to download, or else it will refuse to be added

#### External Mods

If you want to use files that are not downloadable by ferium, place them in the `user` folder in the output directory. Files here will be copied to the output directory when upgrading.

> **Warning**  
> Profiles using the Quilt mod loader will not copy their user mods, this is because Quilt loads mods from nested directories (such as the user folder) for loader versions above `0.18.1-beta.3`

### Adding Modpacks

- Modrinth Modpacks
  - `ferium modpack add project_id`
  - Where `project_id` is the slug or project ID of the modpack
    - For example, [Fabulously Optimized](https://modrinth.com/modpack/fabulously-optimized) has the slug `fabulously-optimized` and project ID `1KVo5zza`
    - You can find the slug in the website URL (`modrinth.com/modpack/<slug>`), and the project id at the bottom of the left sidebar under 'Technical information'
  - So to add [Fabulously Optimized](https://modrinth.com/modpack/fabulously-optimized), you can run `ferium modpack add fabulously-optimized` or `ferium modpack add 1KVo5zza`
- CurseForge Modpacks
  - `ferium modpack add project_id`
  - Where `project_id` is the project ID of the modpack
    - For example, [Fabulously Optimized](https://www.curseforge.com/minecraft/modpacks/fabulously-optimized) has the project ID `396246`
    - You can find the project ID at the top of the right sidebar under 'About Project'
  - So to add [Fabulously Optimized](https://www.curseforge.com/minecraft/modpacks/fabulously-optimized), you should run `ferium modpack add 396246`

### Upgrading Mods

> **Note**  
> If your output directory is not empty when setting it, ferium will offer to create a backup.
> Please do so if it contains any files you would like to keep.

Now after adding all your mods, run `ferium upgrade` to download all of them to your output directory.
This defaults to `.minecraft/mods`, where `.minecraft` is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher (unless you changed the resources directory).
You can choose to pick a custom output directory during profile creation or [change it later](#configure-1).

If ferium fails to download a mod, it will print its name in red and give the reason. It will continue downloading the rest of the mods and will exit with an error.

> **Warning**  
> When upgrading, any files not downloaded by ferium will be moved to the `.old` folder in the output directory

### Upgrading Modpacks

> **Note**  
> If your output directory's `mods` and `resourcepacks` are not empty when setting it, ferium will offer to create a backup.
> Please do so if it contains any files you would like to keep

Now after adding your modpack, run `ferium modpack upgrade` to download the modpack to your output directory.
This defaults to `.minecraft`, which is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher (unless you changed the resources directory).
You can choose to pick a custom output directory when adding modpacks or [change it later](#configure).

If ferium fails to download a mod, it will print its name in red and give the reason. It will continue downloading the rest of the mods and will exit with an error.

> **Warning**  
> If you choose to install modpack overrides, your existing configs may be overwritten

### Managing Mods

You can see all the mods in your current profile by running `ferium list`. If you want to see more information about them, you can run `ferium list -v` or `ferium list --verbose`.

You can remove any of your mods by running `ferium remove`, selecting the ones you would like to remove by using the space key, and pressing enter once you're done. You can also provide the names or IDs of the mods to remove as arguments.

> **Warning**  
> Mod names with spaces have to be given in quotes (`ferium remove "ok zoomer"`) or the spaces should be escaped (`ferium remove ok\ zoomer`).  
> Mod names and GitHub repository identifiers are case insensitive.

#### Check Overrides

If some mod is compatible with your profile but ferium does not download it, [create an issue](https://github.com/gorilla-devs/ferium/issues/new?labels=bug&template=bug-report.md) if you think it's a bug. You can disable the game version or mod loader checks by using the `--dont-check-game-version` and/or `--dont-check-mod-loader` flags when adding the mod, or manually setting `check_game_version` or `check_mod_loader` to false for the specific mod in the config file.

For example, [Just Enough Items](https://www.curseforge.com/minecraft/mc-mods/jei) does not specify the mod loader for older minecraft versions such as `1.12.2`. In this case, you would add JEI by running `ferium add 238222 --dont-check-mod-loader` so that the mod loader check is disabled.
You can also manually disable the mod loader (and/or game version) check(s) in the config like so
```json
{
    "name": "Just Enough Items (JEI)",
    "identifier": {
        "CurseForgeProject": 238222
    },
    "check_mod_loader": false
}
```

### Managing Modpacks

#### Add

When adding a modpack, you will configure the following:

- Output directory
  - This defaults to `.minecraft`, which is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher (unless you changed the resources directory)
- Whether to install modpack overrides

You can also provide these settings as flags.

Ferium will automatically switch to the newly added modpack.

#### Configure

You can configure these same settings afterwards by running `ferium modpack configure`.
Again, you can provide these settings as flags.

#### Manage

You can see all the modpacks you have configured by running `ferium modpack list`.
Switch between your modpacks using `ferium modpack switch`.
Delete a modpack by running `ferium modpack delete` and selecting the modpack you want to delete.

### Profiles

#### Create

You can create a profile by running `ferium profile create` and configuring the following:

- Output directory
  - This defaults to `.minecraft/mods` where `.minecraft` is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher (unless you changed the resources directory)
- Name of the profile
- Minecraft version
- Mod loader

You can also provide these settings as flags.

If you want to copy the mods from another profile, provide the `--import` flag.
You can also directly provide the profile name to the flag if you don't want a profile picker to be shown.

Ferium will automatically switch to the newly created profile.

#### Configure

You can configure these same settings afterwards by running `ferium profile configure`.
Again, you can provide these settings as flags.

#### Manage

You can see all the profiles you have by running `ferium profile list`.
Switch between your profiles using `ferium profile switch`.
Delete a profile by running `ferium profile delete` and selecting the profile you want to delete.

## Feature Requests

If you would like to make a feature request, check the [issue tracker](https://github.com/gorilla-devs/ferium/issues) to see if the feature has already been added or is planned.
If not, [create a new issue](https://github.com/gorilla-devs/ferium/issues/new/choose).

## Building from Source or Working on ferium

Firstly you need the Rust toolchain, which includes `cargo`, `rustup`, etc. You can install these from [the Rust website](https://www.rust-lang.org/tools/install).
You can manually run cargo commands, but I recommend [`just`](https://just.systems/man/en/chapter_4.html), a command runner that is basically a much better version of `make`.

To build the project and install it to your Cargo binary directory, clone the project and run `just install`.
If you want to install it for testing purposes run `just` (alias to `just install-dev`), which builds in debug mode.

You can run integration tests using `cargo test`, linters using `cargo clippy`, and delete all build and test artifacts using `just clean`.

If you would like to see instructions for building for specific targets (e.g. Linux ARM), have a look at the [workflow file](.github/workflows/build.yml). If you're still confused, [create a discussion](https://github.com/gorilla-devs/ferium/discussions/new?category=q-a) and I will help you out.
