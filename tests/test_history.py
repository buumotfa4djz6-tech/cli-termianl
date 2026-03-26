"""Tests for command history management."""

import json
import tempfile
import pytest
from pathlib import Path
from unittest.mock import patch

from cli_terminal.input.history import HistoryManager


class TestHistoryManager:
    """Test history manager."""

    def test_add_command(self):
        """Test adding a command to history."""
        manager = HistoryManager(max_size=10)
        manager.add("test command")
        assert "test command" in manager.history
        assert manager.history[-1] == "test command"

    def test_max_size_limit(self):
        """Test history respects max size limit."""
        manager = HistoryManager(max_size=3)
        manager.add("cmd1")
        manager.add("cmd2")
        manager.add("cmd3")
        manager.add("cmd4")
        assert len(manager.history) == 3
        assert "cmd1" not in manager.history
        assert "cmd4" in manager.history

    def test_search_returns_matches(self):
        """Test search returns matching commands."""
        manager = HistoryManager(max_size=10)
        manager.add("set curr 1.0")
        manager.add("set volt 5.0")
        manager.add("read volt")
        matches = manager.search("volt")
        assert len(matches) == 2
        assert "set volt 5.0" in matches
        assert "read volt" in matches

    def test_search_empty_returns_all(self):
        """Test empty search returns all history."""
        manager = HistoryManager(max_size=10)
        manager.add("cmd1")
        manager.add("cmd2")
        matches = manager.search("")
        assert len(matches) == 2

    def test_persist_to_file(self, tmp_path):
        """Test history persists to JSON file."""
        history_file = tmp_path / "history.json"
        manager = HistoryManager(max_size=10, persist_file=str(history_file))
        manager.add("cmd1")
        manager.add("cmd2")
        manager.save()

        with open(history_file) as f:
            data = json.load(f)
        assert data["commands"] == ["cmd1", "cmd2"]

    def test_load_from_file(self, tmp_path):
        """Test history loads from JSON file."""
        history_file = tmp_path / "history.json"
        with open(history_file, "w") as f:
            json.dump({"commands": ["cmd1", "cmd2"]}, f)

        manager = HistoryManager(max_size=10, persist_file=str(history_file))
        manager.load()
        assert manager.history == ["cmd1", "cmd2"]
