"""Tests for command template functionality."""

import pytest
from cli_terminal.input.templates import TemplateManager, Template


class TestTemplate:
    """Test template parsing."""

    def test_extract_params(self):
        """Test extracting parameter names from template."""
        template = Template("set {param}={value}")
        assert template.params == ["param", "value"]

    def test_no_params(self):
        """Test template with no parameters."""
        template = Template("simple command")
        assert template.params == []

    def test_duplicate_params(self):
        """Test duplicate parameters are deduplicated."""
        template = Template("copy {src} to {dest} then {src} again")
        assert template.params == ["src", "dest"]


class TestTemplateManager:
    """Test template manager."""

    def test_get_template_by_label(self):
        """Test getting template by label."""
        manager = TemplateManager([
            {"label": "Set Param", "command": "set {param}={value}"},
        ])
        template = manager.get("Set Param")
        assert template is not None
        assert template.command == "set {param}={value}"

    def test_get_unknown_returns_none(self):
        """Test unknown label returns None."""
        manager = TemplateManager([])
        assert manager.get("Unknown") is None

    def test_expand_template(self):
        """Test expanding template with values."""
        manager = TemplateManager([])
        template = Template("set {param}={value}")
        result = manager.expand(template, {"param": "curr", "value": "1.0"})
        assert result == "set curr=1.0"
