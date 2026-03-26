"""Tests for search functionality."""

import pytest
from cli_terminal.output.search import SearchManager


class TestSearchManager:
    """Test search manager."""

    def test_set_query_finds_matches(self):
        """Test setting query finds all matches."""
        manager = SearchManager()
        lines = ["ERROR in module", "WARN something", "OK done"]
        manager.set_query("ERROR")
        for line in lines:
            manager.add_line(line)

        matches = manager.get_matches()
        assert len(matches) == 1
        assert matches[0] == (0, "ERROR in module")

    def test_navigate_next(self):
        """Test navigating to next match."""
        manager = SearchManager()
        manager.add_line("ERROR one")
        manager.add_line("normal line")
        manager.add_line("ERROR two")
        manager.set_query("ERROR")

        assert manager.current_index is None
        manager.next_match()
        assert manager.current_index == 0
        manager.next_match()
        assert manager.current_index == 2

    def test_navigate_prev(self):
        """Test navigating to previous match."""
        manager = SearchManager()
        manager.add_line("ERROR one")
        manager.add_line("ERROR two")
        manager.set_query("ERROR")

        manager.next_match()
        manager.next_match()
        assert manager.current_index == 1
        manager.prev_match()
        assert manager.current_index == 0

    def test_clear_search(self):
        """Test clearing search."""
        manager = SearchManager()
        manager.add_line("ERROR")
        manager.set_query("ERROR")
        manager.next_match()

        manager.clear()
        assert manager.query is None
        assert manager.current_index is None
