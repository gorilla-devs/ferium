# Ferium

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&style=for-the-badge&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org/)
[![license badge](https://img.shields.io/github/license/theRookieCoder/ferium?style=for-the-badge)](https://github.com/theRookieCoder/ferium/blob/main/LICENSE.txt)
[![copyleft badge](https://img.shields.io/static/v1?label=&message=Copyleft&style=for-the-badge&labelColor=silver&color=silver&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAyCAQAAAC0NkA6AAAABGdBTUEAALGPC/xhBQAAACBjSFJNAAB6JgAAgIQAAPoAAACA6AAAdTAAAOpgAAA6mAAAF3CculE8AAAAAmJLR0QA/4ePzL8AAAAHdElNRQfjAxYBNgYPa+9oAAAEM0lEQVRYw6WYb0zVVRjHP9wQW7umA0xoKNSC+6bSNkzetKZbaVu19aLpfOGcbcw/S+uNbikuNwMsVyE3XVsro7VEXjS3ylmLxkRtC9crHGjCAv9AATK4CoZye8Hl/J7n/M7v8rvX57w55/lznt/583yf5/xyCEOlrKaSCp6ggCiQYJheLvMHv9HHA1MZ++kmmaZ1UUNZ9g6eo4X7aR3Mtvs0syJzB0U0MR3KgddOsiQTFxsZzdDBTLvFetd0OT5OHo1U+7j9tNJBN4MkgChFVLCS1Sz1aR7jHf5Lv4Yov1hfN8YRKgP1V9LIuGVxhmg6Fwv4XalPcJD8OTe3gA+YVHYXgt3kWato46nQp1jOWWs1eW7Fz5VaLbkZ3cdc6pX9UfeNkvd+a1aRtV3Fle+mLeGWEO/0mT/EWo7SxhBjjNDPKfbxtMPNVjHLKMVa+I0Q1lmG89nDTWdctPGqz80hIT+uAWRaGOqzeJEraQOw2YrzXNqNbJrlnqDFsCeJKZO3uDtnnN+wNq6cCSM74SGtd1wHlfrOkHAyyDPKrk5codIZ1n7DSlAoVF9iKjRq/cVCYZnPmJHsnWF1GcYRobiQf3yA3sr7VPM2cXp9br5Va2k0/EsAy4SixKh6a5LT6rQibGBAyaeV9SohWQabzeBvhUcTaoqPHHhdTKfSOaWk1wx/E8TN4CuhssW6pjnOCF/KiNrOxULWZPgNEbEJF4VKFT2mdbGLpNNJPzVqC9eKkTdbDK4ajy9ngVaPiHuU5AshWWe4VyIsMuwbWTi5Q7sYlYj+TdNbFBHpJZEV8vao8sOjMS8VRh64MkumrRhSh5UQ+T278s+jQdF/1PTGI4yaweNZuHiYF1RsyCiapdFcengyNajgZyP4RBhP8RpDAU42KcxqE30vNK7KYJQpploFY1NgnfmvApYiZxpskLAi6/PFVh454HBRyJ9K5yclvS5hJQggP7YA8vvZzJCi1+m3NKoUYnj8Eg31jSonDFuTTPEju9nIZuq55IP6FvUJ3iF0zjBqApLWOu6FTlp9FCgM90rX9/zpt1Z9z56QLkasatnLRfe8TT5pmHetQqI6RAoesB5A5aIy/s5jrxAl0VmrJHqFvrQuflCwCPM4Jy71s1L0tTA75IPzAyo5ea3D8eg5LORf2mWqnGaXz3Q+b3CcDm6nCtBfqeV5R+xsUyf1mC3eoBLp9qzAcocquN90qRxTW/Fhxk+Hw8o+HvQIOqPU2qkI7SLGeauAmhf8YrygVCepU0HmpkLqLaQ7nz43Ra3VJBknzqpA/SrivofpaduF64n9Kdt83OupJ/YA48ACiolRyRpHovuMd5kKs8PrA+JirjbsvlFBlE9DyP8qXnQ3+eNiblpOc+gfOCc0gGRGpeyzymq7dbLXSmch/q24qIQ1VBKjjMLUT7UheunmIq2qQgmg/wHquM6d9tIV7AAAACV0RVh0ZGF0ZTpjcmVhdGUAMjAxOS0wMy0yMlQwMTo1NDowNiswMDowMOIizoUAAAAldEVYdGRhdGU6bW9kaWZ5ADIwMTktMDMtMjJUMDE6NTQ6MDYrMDA6MDCTf3Y5AAAAAElFTkSuQmCC)](https://en.wikipedia.org/wiki/Copyleft)

> Check out Ferium's sister projects [Ferinth](https://github.com/theRookieCoder/ferinth) and [Furse](https://github.com/theRookieCoder/furse) which are Rust libraries to use the Modrinth and CurseForge APIs respectively

Ferium is an easy to use CLI program for downloading and updating Minecraft mods from [Modrinth](https://modrinth.com/mods), [CurseForge](https://curseforge.com/minecraft/mc-mods), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases). Simply specify the mods you use through the CLI and in just one command, you can download all the mods you configured.

## Features

- Download mods from multiple sources, namely [Modrinth](https://modrinth.com/mods), [CurseForge](https://curseforge.com/minecraft/mc-mods), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- Upgrade all your mods in one command, `ferium upgrade`
  - Ferium checks that the version being downloaded is the latest one compatible with the chosen mod loader and Minecraft version
- Create multiple profiles and configure different mod loaders, Minecraft versions, output directories, and mods for each

## Installation

Ferium is a compiled program that does not require any external dependencies

### GitHub Releases

1. Download the asset suitable for your operating system from [the latest release](https://github.com/theRookieCoder/ferium/releases/latest)
2. Unzip the file and move it to a folder in your path such as `~/bin`
3. Remember to check on the releases for any updates!

### Cargo Install

You can also compile and install Ferium by running `cargo install ferium` if you have the Rust toolchain installed.

Remember to use an add-on like [cargo-update](https://crates.io/crates/cargo-update) to keep Ferium updated to the latest version!

## Overview / Help Page

### First Startup

When you first start up, you will have to create a new profile by running  `ferium profile create` and entering the details for your profile.

### Adding Mods

- Modrinth Mods
  - `ferium add-modrinth project_id`
  - Where `project_id` is the slug or project id of a mod
    - For example, [Sodium](https://modrinth.com/mod/sodium) has slug `sodium` and project id `AANobbMI`
    - You can find the slug in the website url (`modrinth.com/mod/<slug>`) and the project id at the bottom of the left sidebar under 'Technical information'
  - So, to add [Sodium](https://modrinth.com/mod/sodium) to your profile you should run `ferium add-modrinth sodium` or `ferium add-modrinth AANobbMI`
- CurseForge Mods
  - `ferium add-curseforge project_id`
  - Where `project_id` is the project id of the mod
    - For example, [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith) has project id `513688`
    - You can find the project id at the top of the right sidebar under 'About Project'
  - So, to add [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith) you should run `ferium add-curseforge 513688`
- GitHub 'Mods'
  - `ferium add-github owner name`
  - Where `owner` is the username of the owner of the repository and `name` is the name of the repository (both case-insensitive)
    - For example [Sodium's repository](https://github.com/CaffeineMC/sodium-fabric) has the id `CaffeineMC` and `sodium-fabric`
    - You can find these at the top left part of the repository's page as a big 'owner / name'
  - So, to add [Sodium](https://github.com/CaffeineMC/sodium-fabric) you should run `ferium add-github CaffeineMC sodium-fabric` (again, case-insensitive)

### Upgrading Mods

> Warning: upgrading will empty your output directory before downloading mods

Now after adding all your mods, run `ferium upgrade` to download all of them to your output directory. This defaults to `.minecraft/mods` where `.minecraft` is the default Minecraft resources directory, you don't need to worry about this if you play with Mojang's launcher (unless you change the resources directory, of course). You can choose to pick a custom output directory during profile creation or [change it later](#profiles).

If Ferium fails to find a compatible version of a mod, it will print it's name in red and give a reason. It will continue downloading the rest of the mods and will exit with an error. This most likely means that the mod does not release for the mod loader and/or game version you selected. (if it does and Ferium can't find it for some reason [file a bug](https://github.com/theRookieCoder/ferium/issues/new))

PS: [There is a known bug about this](https://github.com/theRookieCoder/ferium/issues/12)

### Managing Mods

You can see all the mods in your current profile by running `ferium list`. If you want to see more information about them, you can run `ferium list -v` or `ferium list --verbose`. You can remove some of your mod by runnning `ferium remove` and selecting the ones you would like to remove by using the space key and pressing enter once you're done.

### Profiles

#### Create
You can create a profile by running `ferium profile create` and configuring the following settings:

- Output directory
  - This defaults to `.minecraft/mods` where `.minecraft` is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher (unless you explicitly change the resources directory)
- Name of the profile
- The Minecraft version
- The mod loader

#### Configure

You can configure these same settings afterwards by running `ferium profile configure`

#### Manage

You can see all the profiles you have by running `ferium 
profile list`. Switch between your profiles using `ferium profile switch`.

#### Delete

Finally, you can delete a profile by running `ferium profile delete` and selecting the profile you want to delete.

## Feature Requests

If you would like to make a feature request, check the [issues](https://github.com/theRookieCoder/ferium/issues?q=is%3Aissue) to see if the feature has already been added/planned. If not, [create a new issue](https://github.com/theRookieCoder/ferium/issues/new).

## Building from Source or Working with Ferium

> Note; A lot of Ferium's backend is in a seperate project, [Libium](https://github.com/theRookieCoder/libium). You might want to make some edits there for things like the config, add, upgrade, etc

Firstly you need the Rust toolchain (`cargo`, `rustup`, etc), you can install these from [the Rust website](https://www.rust-lang.org/tools/install). You'll also need the [Just](https://github.com/casey/just#installation) command runner, its like `make` but better.

If you want to build Ferium without cloning the repo, set the `CURSEFORGE_API_KEY` environment variable then run `cargo install ferium`. If you don't have a CurseForge API key you can set the variable to an empty value, however anything using the CurseForge API will not work.

To build the project and install it to your Cargo binary directory, clone the project then run `just install`. If you want to install for testing a developement version, run `just` (alias for `just install-dev`).

If you want to obtain executables for a specific OS, you can run `just build-<OS>` and replace `<OS>` with `mac`, `win`, or `linux`. The produced binaries will be zipped and moved to `out/`.

You can run clippy linters using `just lint`, and integration tests using `cargo test`.
