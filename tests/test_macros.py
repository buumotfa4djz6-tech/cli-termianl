"""Tests for macro functionality."""

import pytest
from cli_terminal.input.macros import MacroManager


class TestMacroManager:
    """Test macro manager."""

    def test_get_macro_returns_command(self):
        """Test getting macro returns configured command."""
        manager = MacroManager({"F2": "clear", "F3": "restart --force"})
        assert manager.get("F2") == "clear"
        assert manager.get("F3") == "restart --force"

    def test_get_unknown_key_returns_none(self):
        """Test unknown key returns None."""
        manager = MacroManager({"F2": "clear"})
        assert manager.get("F99") is None

    def test_has_key_checks_existence(self):
        """Test has_key checks macro existence."""
        manager = MacroManager({"F2": "clear"})
        assert manager.has_key("F2") is True
        assert manager.has_key("F3") is False
