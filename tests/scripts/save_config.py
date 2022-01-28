from pathlib import Path
import os

config_file = Path.home() / ".config" / "ferium" / "config.json"
config_backup = Path.home() / ".config" / "ferium" / "config_backup.json"

# Save config to backup
os.rename(config_file, config_backup)
