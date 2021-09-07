pub const README: &str = "
Ferium is an open source and easy to use package manager for Minecraft mods (that are on https://modrinth.com and Github Releases).

USAGE
    `ferium <command> [arguments]`
    
COMMANDS
    - `list`
        - Lists all the mods configured with some of their metadata
    - `help`
        - Shows this help page
    - `upgrade`
        - Downloads and installs the latest version of the mods specified
    - `add MOD_ID`
        - Adds a mod to the config
        - A mod's `MOD_ID` is specified as '`</>` PROJECT ID' in the right sidebar of the mod's Modrith page
    - `add-repo OWNER REPO`
        - Adds a repository to the config
        - `OWNER` is username of the owner of the repository
        - `REPO` is the name of the repository
    
EXAMPLES
    # Upgrade all the mods in your config:
    $ ferium upgrade

    # Add the Sodium mod to your config:
    $ ferium add AANobbMI

    # Add the C2ME mod's repository to your config:
    $ ferium add-repo ishlandbukkit C2ME-fabric

ERROR CODES
    - `120`: I/O error
    - `122`: JSON error
    - `124`: Server/HTTP error
    - `126`: General error

FEEDBACK
    You can open and issue here: https://github.com/theRookieCoder/ferium/issues/new.
    If you know how to and are willing to fix it, then you can make a pull request.
    
CONTRIBUTING
    Think you can improve Ferium? Well head on to Ferium's repository (https://github.com/theRookieCoder/ferium) and you can start working on Ferium!
";
