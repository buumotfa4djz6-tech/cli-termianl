"""Configuration loading and validation."""

import os
import yaml
from pathlib import Path
from typing import Any, Dict
from cli_terminal.config.defaults import (
    DEFAULT_CONFIG,
    CONFIG_DIR,
    CONFIG_FILE,
    HISTORY_FILE,
)


class ConfigManager:
    """Manages application configuration."""

    def __init__(self):
        self._config: Dict[str, Any] = DEFAULT_CONFIG.copy()
        self._config_dir = Path(CONFIG_DIR).expanduser()
        self._config_path = self._config_dir / CONFIG_FILE

    def load(self) -> Dict[str, Any]:
        """Load configuration from file, merging with defaults."""
        self._config_dir.mkdir(parents=True, exist_ok=True)

        if not self._config_path.exists():
            self._save_defaults()
            return self._config.copy()

        try:
            with open(self._config_path, "r", encoding="utf-8") as f:
                user_config = yaml.safe_load(f) or {}
            self._merge_config(user_config)
        except yaml.YAMLError as e:
            print(f"Warning: Invalid YAML, using defaults: {e}")

        return self._config.copy()

    def reload(self) -> Dict[str, Any]:
        """Reload configuration from file."""
        self._config = DEFAULT_CONFIG.copy()
        return self.load()

    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        keys = key.split(".")
        value = self._config
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        return value

    def _merge_config(self, user_config: Dict[str, Any]) -> None:
        """Merge user configuration with defaults."""
        for key, value in user_config.items():
            if isinstance(value, dict) and key in self._config:
                if isinstance(self._config[key], dict):
                    self._config[key].update(value)
                else:
                    self._config[key] = value
            else:
                self._config[key] = value

    def _save_defaults(self) -> None:
        """Save default configuration file."""
        with open(self._config_path, "w", encoding="utf-8") as f:
            yaml.safe_dump(DEFAULT_CONFIG, f, default_flow_style=False)

    @property
    def history_path(self) -> Path:
        """Get history file path."""
        return self._config_dir / HISTORY_FILE

    @property
    def config_path(self) -> Path:
        """Get config file path."""
        return self._config_path
