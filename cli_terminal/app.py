"""Main application class."""

import sys
import subprocess
from typing import Optional, List

from prompt_toolkit import Application
from prompt_toolkit.layout import Layout, HSplit, Window
from prompt_toolkit.key_binding import KeyBindings
from prompt_toolkit.shortcuts import input_dialog, radiolist_dialog
from prompt_toolkit.completion import Completer, Completion

from cli_terminal.config.manager import ConfigManager
from cli_terminal.input.history import HistoryManager
from cli_terminal.input.macros import MacroManager
from cli_terminal.input.templates import TemplateManager, Template
from cli_terminal.output.search import SearchManager
from cli_terminal.widgets.display import DisplayWidget
from cli_terminal.widgets.input import InputWidget


class CommandCompleter(Completer):
    """Completer for command input."""

    def __init__(self, commands: dict):
        self._commands = commands

    def get_completions(self, document, complete_event):
        text = document.text_before_cursor
        for label, cmd_str in self._commands.items():
            if label.startswith(text):
                yield Completion(
                    label,
                    start_position=-len(text),
                    display=label,
                    display_meta=cmd_str
                )


class TerminalApp:
    """Main terminal application."""

    def __init__(self, target_program: Optional[str] = None):
        self._target_program = target_program
        self._target_proc: Optional[subprocess.Popen] = None

        # Load configuration
        self._config = ConfigManager()
        config_data = self._config.load()

        # Initialize managers
        self._history = HistoryManager(
            max_size=config_data.get("history.max_size", 1000),
            persist_file=str(self._config.history_path),
        )
        self._history.load()

        self._macros = MacroManager(config_data.get("macros", {}))
        self._templates = TemplateManager(config_data.get("templates", []))
        self._search = SearchManager()

        # Create widgets
        self._display = DisplayWidget(
            timestamp_format=config_data.get("timestamp.format", "[%H:%M:%S.%f]"),
        )

        # Create completer with commands from config
        commands = config_data.get("commands", {})
        completer = CommandCompleter(commands) if commands else None
        self._input = InputWidget(completer=completer)

        # Key bindings
        self._kb = self._create_key_bindings()

        # Create application
        self._app = self._create_app()

    def _create_key_bindings(self) -> KeyBindings:
        """Create key bindings."""
        kb = KeyBindings()

        @kb.add("enter")
        def _(event):
            """Enter to submit command."""
            self._send_command(self._input.text)

        @kb.add("c-r")
        async def _(event):
            """Ctrl+R for history search."""
            await self._history_search()

        @kb.add("c-l")
        def _(event):
            """Ctrl+L to reload config."""
            self._reload_config()

        @kb.add("/")
        async def _(event):
            """/ to activate search."""
            await self._activate_search()

        @kb.add("n")
        def _(event):
            """n for next match."""
            self._next_match()

        @kb.add("N")
        def _(event):
            """N for previous match."""
            self._prev_match()

        @kb.add("f1")
        async def _(event):
            """F1 for template menu."""
            await self._show_templates()

        @kb.add("c-c")
        def _(event):
            """Ctrl+C to quit."""
            event.app.exit()

        # Macro bindings F2-F12
        for i in range(2, 13):
            key = f"f{i}"
            command = self._macros.get(f"F{i}")
            if command:
                kb.add(key)(lambda e, c=command: self._execute_macro(c))

        return kb

    def _create_app(self) -> Application:
        """Create prompt_toolkit application."""
        container = HSplit([
            self._display.textarea,
            Window(height=1, char="-"),
            self._input.textarea,
        ])
        return Application(
            layout=Layout(container, focused_element=self._input.textarea),
            key_bindings=self._kb,
            full_screen=True,
            mouse_support=True,
        )

    def _send_command(self, command: str) -> None:
        """Send command to target program."""
        if not command.strip():
            return

        self._history.add(command)
        self._display.add_line(f"> {command}")

        if self._target_proc:
            self._target_proc.stdin.write(command + "\n")
            self._target_proc.stdin.flush()
        else:
            print(command, flush=True)

        self._input.clear()

    def _execute_macro(self, command: str) -> None:
        """Execute a macro command."""
        self._send_command(command)

    async def _history_search(self) -> None:
        """Activate history search with dialog."""
        # Get search query
        query = await input_dialog(
            title="History Search",
            text="Enter search query:",
        )

        if query:
            matches = self._history.search(query)
            if not matches:
                self._display.add_line("[No matches found]")
                return

            # Show matches in radiolist
            choices = [(m, m) for m in matches]
            result = await radiolist_dialog(
                title="Select Command",
                text="Choose a command:",
                values=choices,
            )

            if result:
                self._input.text = result

    def _reload_config(self) -> None:
        """Reload configuration."""
        self._config.reload()
        self._display.add_line("[Config reloaded]")

    async def _activate_search(self) -> None:
        """Activate output search with input dialog."""
        query = await input_dialog(
            title="Search",
            text="Search query:",
        )
        if query:
            self._search.set_query(query)
            self._next_match()

    def _next_match(self) -> None:
        """Go to next search match."""
        self._search.next_match()

    def _prev_match(self) -> None:
        """Go to previous search match."""
        self._search.prev_match()

    async def _show_templates(self) -> None:
        """Show template selection dialog."""
        templates = self._templates.all
        if not templates:
            self._display.add_line("[No templates configured]")
            return

        choices = [(t.label or t.command, t) for t in templates]
        result = await radiolist_dialog(
            title="Select Template",
            text="Choose a template:",
            values=choices,
        )

        if result:
            template = result
            if template.params:
                # Prompt for each parameter
                values = {}
                for param in template.params:
                    prompt_text = template.param_prompts.get(param, param) if template.param_prompts else param
                    value = await input_dialog(
                        title=f"Parameter: {param}",
                        text=f"Enter {prompt_text}:",
                    )
                    if value is None:
                        return  # User cancelled
                    values[param] = value

                command = template.expand(values)
            else:
                command = template.command

            self._send_command(command)

    def start(self) -> None:
        """Start the target program and application."""
        if self._target_program:
            try:
                self._target_proc = subprocess.Popen(
                    [self._target_program],
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    bufsize=1,
                )
                print(f"Connected to: {self._target_program}", file=sys.stderr)
            except FileNotFoundError:
                print(f"Error: Program not found: {self._target_program}", file=sys.stderr)
                return 1

        try:
            self._app.run()
        finally:
            self._cleanup()

    def _cleanup(self) -> None:
        """Clean up resources."""
        self._history.save()
        if self._target_proc:
            self._target_proc.terminate()
            self._target_proc.wait(timeout=5)


def main() -> int:
    """Main entry point."""
    target_program = sys.argv[1] if len(sys.argv) > 1 else None
    app = TerminalApp(target_program)
    app.start()
    return 0
