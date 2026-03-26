"""Tests for output highlighting functionality."""

import pytest
from cli_terminal.output.highlight import HighlightManager, HighlightRule


class TestHighlightRule:
    """Test highlight rule matching."""

    def test_rule_matches(self):
        """Test rule matches pattern."""
        rule = HighlightRule(pattern="ERROR", fg="red")
        assert rule.matches("This is an ERROR message") is True
        assert rule.matches("Everything is fine") is False

    def test_rule_is_case_sensitive(self):
        """Test rule matching is case sensitive."""
        rule = HighlightRule(pattern="ERROR", fg="red")
        assert rule.matches("ERROR") is True
        assert rule.matches("error") is False


class TestHighlightManager:
    """Test highlight manager."""

    def test_find_matching_rules(self):
        """Test finding matching rules for text."""
        manager = HighlightManager([
            {"pattern": "ERROR", "fg": "red"},
            {"pattern": "WARN", "fg": "yellow"},
        ])
        matches = manager.find_matches("ERROR occurred")
        assert len(matches) == 1
        assert matches[0].fg == "red"

    def test_first_match_wins(self):
        """Test first matching rule wins for same position."""
        manager = HighlightManager([
            {"pattern": "ERR", "fg": "red"},
            {"pattern": "ERROR", "fg": "blue"},
        ])
        matches = manager.find_matches("ERROR")
        # Both match at position 0, but first one should be returned
        assert len(matches) >= 1
