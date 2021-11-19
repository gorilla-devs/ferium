import os

config_file = os.path.expanduser("~/.config/ferium/config.json")
config_backup = os.path.expanduser("~/.config/ferium/config_backup.json")

# Save config to backup
os.rename(config_file, config_backup)
# Write default data to new testing config
config = open(config_file, 'x')
config.write("""{
  "output_dir": "/Users/ilesh/Library/ApplicationSupport/minecraft/mods",
  "game_version": "1.17.1",
  "mod_loader": "fabric",
  "mod_ids": [],
  "repos": []
}""")
config.close()
