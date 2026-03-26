"""Tests for configuration management."""

import os
import tempfile
import pytest
from pathlib import Path
from unittest.mock import patch, MagicMock

from cli_terminal.config.manager import ConfigManager
from cli_terminal.config.defaults import DEFAULT_CONFIG


class TestConfigManager:
    """Test configuration manager."""

    def test_load_creates_defaults_if_missing(self, tmp_path):
        """Test that loading creates default config if file missing."""
        manager = ConfigManager()
        manager._config_dir = tmp_path
        manager._config_path = tmp_path / "config.yaml"
        config = manager.load()
        assert config == DEFAULT_CONFIG

    def test_get_nested_value(self, tmp_path):
        """Test getting nested configuration values."""
        manager = ConfigManager()
        manager._config = {"timestamp": {"enabled": True, "format": "[%H:%M:%S]"}}
        assert manager.get("timestamp.enabled") is True
        assert manager.get("timestamp.format") == "[%H:%M:%S]"

    def test_get_missing_key_returns_default(self):
        """Test getting missing key returns default."""
        manager = ConfigManager()
        assert manager.get("nonexistent.key", "default") == "default"
        assert manager.get("nonexistent.key") is None

    def test_reload_refreshes_config(self, tmp_path):
        """Test reload refreshes configuration."""
        manager = ConfigManager()
        manager._config = {"custom": "value"}
        reloaded = manager.reload()
        assert reloaded == DEFAULT_CONFIG
