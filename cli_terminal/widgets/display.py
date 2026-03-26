"""Main display widget for output."""

from typing import List, Optional
from prompt_toolkit.widgets import TextArea
from prompt_toolkit.formatted_text import FormattedText


class DisplayWidget:
    """Enhanced display widget with highlighting and timestamps."""

    def __init__(self, timestamp_format: str = "[%H:%M:%S.%f]"):
        self._timestamp_format = timestamp_format
        self._lines: List[str] = []
        self._textarea = TextArea(
            text="",
            read_only=True,
            wrap_lines=True,
        )

    def add_line(self, line: str, formatted: bool = False) -> None:
        """Add a line to display."""
        self._lines.append(line)
        self._update_display()

    def _update_display(self) -> None:
        """Update the textarea with current lines."""
        self._textarea.text = "\n".join(self._lines)

    def clear(self) -> None:
        """Clear display."""
        self._lines = []
        self._update_display()

    @property
    def textarea(self):
        """Get underlying textarea."""
        return self._textarea
