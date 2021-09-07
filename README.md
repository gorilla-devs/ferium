# Ferium

Ferium is an open source and easy to use package manager for Minecraft mods (that are on [Modrinth](https://modrinth.com) and [Github](https://github.com/releases) Releases).

## Feature to do

- [x] Allow configuration through a CLI
- [x] Add support for downloading from Github releases
- [ ] Release the Labrinth library as a seperate Crate
- [ ] Add support for checking mod loader compatibility
- [ ] Check that the version is compatible with a specified Minecraft version
- [ ] Add support for downloading from CurseForge

## Help page

This section is the same as the one in `ferium help` + formatting.

### Usage
`ferium <command> [arguments]`
    
### Commands
- `list`
  - Lists all the mods configured with some of their metadata
- `help`
  - Shows this help page
- `upgrade`
  - Downloads and installs the latest version of the mods specified
- `add MOD_ID`
  - Adds a Modrinth mod to the config
  - A mod's `MOD_ID` is specified as '`</>` PROJECT ID' in the right sidebar of the mod's Modrith page
- `add-repo OWNER REPO`
  - Adds a repository to the config
  - `OWNER` is username of the owner of the repository
  - `REPO` is the name of the repository
    
### Examples
```bash
# Upgrade all the mods in your config:
$ ferium upgrade

# Add the Sodium mod to your config:
$ ferium add AANobbMI

# Add the C2ME mod's repository to your config:
$ ferium add-repo ishlandbukkit C2ME-fabric
```

### Error codes
- `120`: I/O error
- `122`: JSON error
- `124`: Server/HTTP error
- `126`: General error

### Feedback
You can [open an issue](https://github.com/theRookieCoder/ferium/issues/new).  
If you know how to and are willing to fix it, then you can make a pull request.
    
### Contributing
Think you can improve Ferium? Well head on to [Ferium's repository](https://github.com/theRookieCoder/ferium) and you can start working on Ferium!
