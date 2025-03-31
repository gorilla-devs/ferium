# Libium

> [!IMPORTANT]
> This project used to be in its [own repository](https://github.com/gorilla-devs/libium), but it has now been moved into ferium's repository to make pull requests and dependency syncing easier.  
> You will need to go to the old repository to see the commit history.

Libium is the backend of [ferium](https://github.com/gorilla-devs/ferium). It helps manage Minecraft mods from Modrinth, CurseForge, and Github Releases.

Here's a brief description of the main components of libium;

- `config` defines the config structure and methods to get the config file, deserialise it, upgrade it to a new version, etc.
- `modpack` contains manifest/metadata structs for Modrinth and CurseForge modpack formats, reads these from a zip file, and adds modpacks to configs.
- `upgrade` defines and implements filters, and fetches the latest compatible mod/modpack file, and downloads it.
- `add` verifies and adds a mod to a profile.
- `scan` hashes mod files in a directory and sends them to the Modrinth and CurseForge APIs to retrieve mod information and add them to a profile.
