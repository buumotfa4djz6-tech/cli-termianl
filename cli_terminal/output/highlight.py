"""Output highlighting with keyword and syntax rules."""

import re
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class HighlightRule:
    """A highlighting rule."""
    pattern: str
    fg: Optional[str] = None
    bg: Optional[str] = None
    bold: bool = False
    _compiled: Optional[re.Pattern] = None

    def __post_init__(self):
        self._compiled = re.compile(self.pattern)

    def matches(self, text: str) -> bool:
        """Check if pattern matches text."""
        return self._compiled.search(text) is not None

    def find_spans(self, text: str) -> List[Tuple[int, int]]:
        """Find all match spans in text."""
        return [(m.start(), m.end()) for m in self._compiled.finditer(text)]


@dataclass
class HighlightSpan:
    """A span of highlighted text."""
    start: int
    end: int
    text: str
    fg: Optional[str] = None
    bg: Optional[str] = None
    bold: bool = False


class HighlightManager:
    """Manages highlighting rules and application."""

    def __init__(self, rules_config: List[Dict[str, Any]]):
        self._rules: List[HighlightRule] = []
        for cfg in rules_config:
            self._rules.append(HighlightRule(
                pattern=cfg.get("pattern", ""),
                fg=cfg.get("fg"),
                bg=cfg.get("bg"),
                bold=cfg.get("bold", False),
            ))

    def find_matches(self, text: str) -> List[HighlightRule]:
        """Find all rules that match the text."""
        return [rule for rule in self._rules if rule.matches(text)]

    def apply(self, text: str) -> List[HighlightSpan]:
        """Apply all highlighting rules to text."""
        spans: List[HighlightSpan] = []
        for rule in self._rules:
            for start, end in rule.find_spans(text):
                spans.append(HighlightSpan(
                    start=start,
                    end=end,
                    text=text[start:end],
                    fg=rule.fg,
                    bg=rule.bg,
                    bold=rule.bold,
                ))
        return spans
