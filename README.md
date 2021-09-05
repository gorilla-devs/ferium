# Ferium

Ferium is an open source and easy to use package manager for Minecraft mods (that are on [Modrinth](https://modrinth.com)).

## Feature to do

- [x] Allow configuration through a CLI
- [ ] Add support for downloading from Github releases
- [ ] Release the Labrinth library as a seperate Crate
- [ ] Check that the version is compatible with a specified Minecraft version
- [ ] Add support for downloading from CurseForge

## Help page

This section is the same as the one in `ferium help` + formatting.

### Usage
`ferium <command> [arguments]`
    
### Commands
- `list`
  - List all the mods configured with some of their metadata
- `help`
  - Show this help page
- `upgrade`
  - Download and install the latest version of the mods specified
- `add MOD_ID`
  - Add a mod to the config
  - A mod's `MOD_ID` is specified as '`</>` PROJECT ID' in the right sidebar of the mod's Modrith page
    
### Examples
```bash
$ ferium upgrade        # Upgrades all the mods in your config
$ ferium add AANobbMI   # Adds the Sodium mod to your config
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
