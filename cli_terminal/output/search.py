"""Search functionality for output content."""

import re
from typing import List, Optional, Tuple


class SearchManager:
    """Manages search state and navigation."""

    def __init__(self):
        self._query: Optional[str] = None
        self._compiled: Optional[re.Pattern] = None
        self._lines: List[str] = []
        self._matches: List[Tuple[int, str]] = []
        self._current_match_index: Optional[int] = None

    @property
    def query(self) -> Optional[str]:
        """Get current search query."""
        return self._query

    @property
    def current_index(self) -> Optional[int]:
        """Get current match index."""
        if self._current_match_index is None:
            return None
        return self._matches[self._current_match_index][0]

    @property
    def match_count(self) -> int:
        """Get total match count."""
        return len(self._matches)

    def set_query(self, query: str) -> None:
        """Set search query and find matches."""
        self._query = query if query else None
        self._compiled = re.compile(re.escape(query)) if query else None
        self._current_match_index = None
        self._update_matches()

    def add_line(self, line: str) -> None:
        """Add a line to searchable content."""
        self._lines.append(line)
        if self._compiled:
            if self._compiled.search(line):
                self._matches.append((len(self._lines) - 1, line))

    def clear(self) -> None:
        """Clear search state."""
        self._query = None
        self._compiled = None
        self._matches = []
        self._current_match_index = None

    def next_match(self) -> Optional[int]:
        """Navigate to next match."""
        if not self._matches:
            return None
        if self._current_match_index is None:
            self._current_match_index = 0
        else:
            self._current_match_index = (self._current_match_index + 1) % len(self._matches)
        return self.current_index

    def prev_match(self) -> Optional[int]:
        """Navigate to previous match."""
        if not self._matches:
            return None
        if self._current_match_index is None:
            self._current_match_index = len(self._matches) - 1
        else:
            self._current_match_index = (self._current_match_index - 1) % len(self._matches)
        return self.current_index

    def get_matches(self) -> List[Tuple[int, str]]:
        """Get all matches with line indices."""
        return self._matches.copy()

    def _update_matches(self) -> None:
        """Update matches based on current query."""
        if not self._compiled:
            self._matches = []
            return
        self._matches = [
            (i, line) for i, line in enumerate(self._lines)
            if self._compiled.search(line)
        ]
