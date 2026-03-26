"""Output rendering with timestamps and highlighting."""

from datetime import datetime
from typing import List, Optional
from cli_terminal.output.highlight import HighlightManager, HighlightSpan


class OutputRenderer:
    """Renders output with timestamps and highlighting."""

    def __init__(self, timestamp_format: str = "[%H:%M:%S.%f]", highlight_manager: Optional[HighlightManager] = None):
        self._timestamp_format = timestamp_format
        self._highlight_manager = highlight_manager
        self._lines: List[str] = []

    def add_line(self, line: str, add_timestamp: bool = False) -> None:
        """Add a line to output."""
        if add_timestamp:
            timestamp = datetime.now().strftime(self._timestamp_format)
            line = f"{timestamp} {line}"
        self._lines.append(line)

    def get_lines(self) -> List[str]:
        """Get all lines."""
        return self._lines.copy()

    def clear(self) -> None:
        """Clear all output."""
        self._lines = []
