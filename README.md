# Ferium

Ferium is an open source and easy to use package manager for Minecraft mods (that are on [Modrinth](https://modrinth.com) and [Github](https://github.com/releases) Releases). Simply add the mods you use (through the CLI or config file) and in just one command, download all the mods and update existing ones.

## Feature to do

I hope to implement all the following improvements, preferably in the order listed. No gaurantees though.

- [x] Allow configuration through a CLI
- [x] Add support for downloading from Github releases
- [ ] Add support for checking mod loader and Minecraft version compatibility
- [ ] Release the Labrinth library as a seperate Crate
- [ ] Add support for downloading from CurseForge (Might need help for this one)

## Help page

### Usage
`ferium <command> [arguments]`
    
### Commands
- `list`
  - Lists all the mods/repos configured with some of their metadata
- `help`
  - Shows this help page
- `upgrade`
  - Downloads and installs the latest version of the mods in your config file
- `add MOD_ID`
  - Adds a Modrinth mod to the config
  - Go to [Modrinth](https://modrinth.com/mods?q=) and search for the mod you want to add
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
- `120` I/O error
- `122` JSON error
- `124` Server/HTTP error
- `126` General error

## Feedback
You can [open an issue](https://github.com/theRookieCoder/ferium/issues/new) if you have found a bug, or would like to request a feature.  
If you know how to and are willing to fix it, then you can make a pull request.
    
## Contributing
Think you can improve Ferium? Well head on to [Ferium's repository](https://github.com/theRookieCoder/ferium) and you can start working on Ferium!
