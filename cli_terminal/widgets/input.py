"""Input widget with enhancements."""

from typing import Callable, Optional
from prompt_toolkit.widgets import TextArea
from prompt_toolkit.completion import Completer


class InputWidget:
    """Enhanced input widget with completion and history."""

    def __init__(
        self,
        completer: Optional[Completer] = None,
        on_submit: Optional[Callable[[str], None]] = None,
    ):
        self._on_submit = on_submit
        self._textarea = TextArea(
            height=3,
            prompt="$ ",
            multiline=False,
            wrap_lines=True,
            completer=completer,
        )

    @property
    def text(self) -> str:
        """Get current text."""
        return self._textarea.text

    @text.setter
    def text(self, value: str) -> None:
        """Set current text."""
        self._textarea.text = value

    def clear(self) -> None:
        """Clear input."""
        self._textarea.text = ""

    @property
    def textarea(self):
        """Get underlying textarea."""
        return self._textarea
