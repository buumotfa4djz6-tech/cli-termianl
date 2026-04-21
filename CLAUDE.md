# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this repository.

Refer to README.md for project overview and architecture.

## Development Conventions

- 符合最佳实践
- 优先使用成熟库

## Commit Convention

使用中文提交，保持简短。

## Keybindings

Keybindings are defined in `src/config/defaults.rs`. When adding or changing keybindings, update the defaults and keep them consistent across modes.

## Config Schema

Config is loaded from `~/.config/cli-terminal/config.yaml`. The two-layer `RawConfig` → `AppConfig` pattern means:
- `RawConfig` handles serde deserialization with optional fields
- `AppConfig` is the typed, flattened representation used by the app
- When adding config fields, update both layers and `src/config/defaults.rs`

## Testing

Run `cargo test` to execute tests. Tests live in the `tests/` directory.
