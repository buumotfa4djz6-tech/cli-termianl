"""Command history management with persistence."""

import json
from pathlib import Path
from typing import List, Optional


class HistoryManager:
    """Manages command history with search and persistence."""

    def __init__(self, max_size: int = 1000, persist_file: Optional[str] = None):
        self._max_size = max_size
        self._persist_file = Path(persist_file) if persist_file else None
        self._history: List[str] = []

    @property
    def history(self) -> List[str]:
        """Get current history."""
        return self._history.copy()

    def add(self, command: str) -> None:
        """Add a command to history."""
        if not command.strip():
            return
        self._history.append(command)
        if len(self._history) > self._max_size:
            self._history.pop(0)

    def search(self, query: str) -> List[str]:
        """Search history for matching commands."""
        if not query:
            return self._history.copy()
        query_lower = query.lower()
        return [cmd for cmd in self._history if query_lower in cmd.lower()]

    def save(self) -> None:
        """Save history to file."""
        if not self._persist_file:
            return
        self._persist_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self._persist_file, "w", encoding="utf-8") as f:
            json.dump({"commands": self._history}, f, indent=2)

    def load(self) -> None:
        """Load history from file."""
        if not self._persist_file or not self._persist_file.exists():
            return
        try:
            with open(self._persist_file, "r", encoding="utf-8") as f:
                data = json.load(f)
            self._history = data.get("commands", [])
        except (json.JSONDecodeError, IOError):
            self._history = []

    def clear(self) -> None:
        """Clear history."""
        self._history = []
        if self._persist_file and self._persist_file.exists():
            self._persist_file.unlink()
