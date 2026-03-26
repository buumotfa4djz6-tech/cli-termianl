"""Macro management for shortcut commands."""

from typing import Dict, Optional


class MacroManager:
    """Manages keyboard macro bindings."""

    def __init__(self, macros: Dict[str, str]):
        self._macros = macros.copy() if macros else {}

    def get(self, key: str) -> Optional[str]:
        """Get command for a key binding."""
        return self._macros.get(key)

    def has_key(self, key: str) -> bool:
        """Check if a key has a macro bound."""
        return key in self._macros
