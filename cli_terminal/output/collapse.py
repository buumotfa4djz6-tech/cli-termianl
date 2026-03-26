"""Region collapsing for output content."""

from dataclasses import dataclass
from typing import List, Optional


@dataclass
class CollapseRegion:
    """A collapsible region."""
    start: int
    end: int
    header: str
    collapsed: bool = False


class CollapseManager:
    """Manages collapsible regions."""

    def __init__(self):
        self._regions: List[CollapseRegion] = []
        self._next_id: int = 0

    def add_region(self, start: int, end: int, header: str = "") -> int:
        """Add a collapsible region."""
        region_id = self._next_id
        self._next_id += 1
        self._regions.append(CollapseRegion(
            start=start,
            end=end,
            header=header or f"Region {region_id}",
        ))
        return region_id

    def toggle(self, index: int) -> bool:
        """Toggle collapse state of region."""
        if 0 <= index < len(self._regions):
            self._regions[index].collapsed = not self._regions[index].collapsed
            return self._regions[index].collapsed
        return False

    def is_collapsed(self, line_index: int) -> Optional[int]:
        """Check if line is in collapsed region, return region index."""
        for i, region in enumerate(self._regions):
            if region.collapsed and region.start <= line_index < region.end:
                return i
        return None

    def get_visible_lines(self, lines: List[str]) -> List[str]:
        """Get list of visible lines (excluding collapsed)."""
        result = []
        for i, line in enumerate(lines):
            collapsed = self.is_collapsed(i)
            if collapsed is None:
                result.append(line)
            elif i == self._regions[collapsed].start:
                result.append(f"[+ {self._regions[collapsed].header}]")
        return result
