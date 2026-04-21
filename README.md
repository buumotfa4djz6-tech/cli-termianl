# cli-terminal

A TUI (Terminal User Interface) that enhances terminal interaction. It wraps a target program (or standalone), providing command history with search, syntax highlighting, templates/macros, and output collapse.

Built with Rust, using `ratatui` for TUI rendering and `crossterm` for terminal control.

## Commands

```bash
cargo build                          # Build the project
cargo run                            # Run without target program
cargo run -- <program>               # Run connected to a target program (e.g. cargo run -- bash)
cargo test                           # Run all tests
cargo check                          # Fast compilation check
```

## Architecture

### Module Structure

The codebase has 5 modules under `src/`:

| Module | Responsibility |
|--------|---------------|
| `app` | Main `App` struct, event loop, terminal lifecycle, child process management, rendering |
| `config` | YAML config loading/parsing (`RawConfig` → `AppConfig`), color deserialization, default values |
| `input` | Command history (`HistoryManager`), F-key macros (`MacroManager`), command templates (`TemplateManager`) |
| `output` | Syntax highlighting (`highlight_line`), regex search (`SearchState`), collapsible regions (`CollapseManager`) |
| `widgets` | Display widget (`DisplayWidget` for output lines), input buffer (`InputBuffer` with Unicode support) |

### Key Data Flows

1. **Child process**: A target program is spawned with piped stdin/stdout/stderr. stdout and stderr are read on separate threads and sent through a `crossbeam_channel` to the main event loop, which drains them into `DisplayWidget`.

2. **Event loop**: Polls for keyboard input every 50ms, dispatching to mode-specific handlers (`Normal`, `HistorySearch`, `OutputSearch`, `TemplateMenu`). Normal mode handles editing, submission (sends to child stdin), and mode transitions.

3. **Config**: Loaded from `~/.config/cli-terminal/config.yaml`. On startup, default config is written if file doesn't exist. `Ctrl+L` reloads config at runtime.

### Modes

- **Normal**: Command editing, submit with Enter, Ctrl+R for history search, `/` for output search, F1 for template menu, F2-F12 for macros, Ctrl+E to export output, Ctrl+L to reload config
- **HistorySearch**: Filter history with character-by-character query, navigate with Up/Down
- **OutputSearch**: Regex search over displayed output, `n`/`N` for next/previous match
- **TemplateMenu**: Select from configured templates, suspend terminal for parameter prompting

### Notable Details

- `InputBuffer` uses `unicode-segmentation` for proper grapheme cluster handling
- Highlight rules are applied per-byte using regex; first matching rule wins
- Config uses a two-layer approach: `RawConfig` (serde-deserializable) → `AppConfig` (typed, flattened)
- The old Python codebase was removed (see git history for reference)
