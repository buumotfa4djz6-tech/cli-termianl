# CLI Terminal

Universal stdin/stdout interaction enhancer.

## Installation

```bash
pip install -r requirements.txt
```

## Usage

```bash
# Direct program connection
python -m cli_terminal ./your-program

# Or pipe input
echo "command" | python -m cli_terminal
```

## Configuration

Configuration file: `~/.config/cli-terminal/config.yaml`

See `docs/superpowers/specs/2026-03-26-cli-terminal-design.md` for full configuration options.

## Key Bindings

| Key | Action |
|-----|--------|
| F1 | Template menu |
| F2-F12 | Macro |
| Ctrl+R | History search |
| Ctrl+L | Reload config |
| / | Search output |
| n/N | Next/Previous match |
| Tab | Toggle collapse |
| Enter | Execute command |
| Ctrl+C | Quit |

## Features

- **Command History** - Persistent history with search (Ctrl+R)
- **Macros** - Shortcut commands on F2-F12
- **Templates** - Parameterized commands with prompts
- **Output Highlighting** - Keywords like ERROR, WARN, OK highlighted
- **Search** - Search output with / key, navigate with n/N
- **Configuration** - YAML config file with hot-reload (Ctrl+L)
