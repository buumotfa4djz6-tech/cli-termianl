"""Default configuration values."""

DEFAULT_CONFIG = {
    "commands": {},
    "macros": {
        "F2": "clear",
    },
    "templates": [],
    "highlights": [
        {"pattern": "ERROR", "fg": "red", "bg": "black", "bold": True},
        {"pattern": "WARN", "fg": "yellow", "bg": "black"},
        {"pattern": "OK|SUCCESS", "fg": "green", "bg": "black"},
    ],
    "syntax": {},
    "timestamp": {
        "enabled": True,
        "format": "[%H:%M:%S.%f]",
    },
    "history": {
        "max_size": 1000,
        "persist": True,
    },
}

CONFIG_DIR = "~/.config/cli-terminal"
CONFIG_FILE = "config.yaml"
HISTORY_FILE = "history.json"
