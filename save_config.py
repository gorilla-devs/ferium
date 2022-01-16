import os

config_file = os.path.expanduser("~/.config/ferium/config.json")
config_backup = os.path.expanduser("~/.config/ferium/config_backup.json")

# Save config to backup
os.rename(config_file, config_backup)
