"""Command template management with parameter expansion."""

import re
from typing import Dict, List, Optional
from dataclasses import dataclass


@dataclass
class Template:
    """A command template with parameters."""
    command: str
    label: Optional[str] = None
    param_prompts: Optional[Dict[str, str]] = None

    @property
    def params(self) -> List[str]:
        """Extract parameter names from command."""
        matches = re.findall(r"\{(\w+)\}", self.command)
        seen = set()
        result = []
        for p in matches:
            if p not in seen:
                seen.add(p)
                result.append(p)
        return result

    def expand(self, values: Dict[str, str]) -> str:
        """Expand template with given values."""
        result = self.command
        for param, value in values.items():
            result = result.replace("{" + param + "}", value)
        return result


class TemplateManager:
    """Manages command templates."""

    def __init__(self, templates_config: List[Dict]):
        self._templates: List[Template] = []
        for cfg in templates_config:
            label = cfg.get("label", "")
            command = cfg.get("command", "")
            prompts = {}
            for param in cfg.get("params", []):
                prompts[param["name"]] = param.get("prompt", param["name"])
            self._templates.append(Template(
                command=command,
                label=label,
                param_prompts=prompts if prompts else None,
            ))

    def get(self, label: str) -> Optional[Template]:
        """Get template by label."""
        for t in self._templates:
            if t.label == label:
                return t
        return None

    @property
    def all(self) -> List[Template]:
        """Get all templates."""
        return self._templates.copy()

    def expand(self, template: Template, values: Dict[str, str]) -> str:
        """Expand a template with values."""
        return template.expand(values)
