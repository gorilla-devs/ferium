import os

config_file = os.path.expanduser("~/.config/ferium/config.json")
config_backup = os.path.expanduser("~/.config/ferium/config_backup.json")

# Remove testinf config file
os.remove(config_file)
# Replace with backup
os.rename(config_backup, config_file)
