from pathlib import Path
import os

config_file = Path.home() / ".config" / "ferium" / "config.json"
config_backup = Path.home() / ".config" / "ferium" / "config_backup.json"

# Remove testing config file
try:
    os.remove(config_file)
except Exception:
    pass
# Replace with backup
os.rename(config_backup, config_file)
