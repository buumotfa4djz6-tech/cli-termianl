# cli-terminal — justfile
# https://github.com/casey/just

# Default: show available recipes
default:
    @just --list

# ── Build ──────────────────────────────────────────────────────────

# Build the project
build:
    cargo build

# Release build
release:
    cargo build --release

# Fast check-only build
check:
    cargo check

# ── Run ────────────────────────────────────────────────────────────

# Run without target program
run:
    cargo run

# Run connected to a target program
run-to target:
    cargo run -- {{target}}

# Release-optimized run
run-release:
    cargo run --release

# ── Lint ───────────────────────────────────────────────────────────

# Check formatting (no changes)
fmt-check:
    cargo fmt -- --check

# Auto-format code
fmt:
    cargo fmt

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# Full lint: fmt + clippy
lint: fmt-check && clippy

# ── Test ───────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test

# Run a single test by name
test-one name:
    cargo test {{name}}

# Run tests, showing stdout output
test-verbose:
    cargo test -- --nocapture

# ── Fix ────────────────────────────────────────────────────────────

# Format + fix clippy warnings + build
fix: fmt clippy build

# ── Clean ──────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean
