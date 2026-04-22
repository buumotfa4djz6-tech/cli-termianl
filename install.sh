#!/usr/bin/env bash
# cli-terminal installer
# Usage: curl -fsSL https://raw.githubusercontent.com/buumotfa4djz6-tech/cli-terminal/refs/heads/master/install.sh | bash

set -euo pipefail

BINARY="cli-terminal"
REPO="buumotfa4djz6-tech/cli-terminal"
GITHUB="https://github.com"

# ── Colors ──────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()    { echo -e "${GREEN}[info]${NC} $*"; }
warn()    { echo -e "${YELLOW}[warn]${NC} $*"; }
error()   { echo -e "${RED}[error]${NC} $*" >&2; }

# ── Detect OS & Arch ────────────────────────────────────────────────
detect_target() {
    local os arch target

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="darwin" ;;
        *)       error "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64)  arch="x86_64" ;;
        arm64|aarch64)
            if [ "$os" = "linux" ]; then arch="aarch64"; else arch="aarch64"; fi
            ;;
        *)       error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    target="${arch}-unknown-${os}"
    if [ "$os" = "darwin" ]; then
        target="${arch}-apple-darwin"
    fi

    echo "$target"
}

# ── Check dependencies ─────────────────────────────────────────────
check_deps() {
    for cmd in curl tar; do
        if ! command -v "$cmd" &>/dev/null; then
            error "'$cmd' is required but not found. Please install it first."
            exit 1
        fi
    done
}

# ── Install from GitHub release ────────────────────────────────────
install_from_release() {
    local target="$1"
    local version="${2:-latest}"
    local api_url download_url tmpdir asset_name

    asset_name="cli-terminal-${target}"

    if [ "$version" = "latest" ]; then
        api_url="${GITHUB}/api/repos/${REPO}/releases/latest"
    else
        api_url="${GITHUB}/api/repos/${REPO}/releases/tags/${version}"
    fi

    info "Fetching release info: ${version}"

    if ! download_url=$(curl -fsSL -H "Accept: application/vnd.github+json" "$api_url" 2>/dev/null \
        | grep -o "${GITHUB}/${REPO}/releases/download/[^/\"]*/${asset_name}\.tar\.gz" | head -1); then
        return 1
    fi

    info "Downloading: ${download_url}"

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -fsSL -o "${tmpdir}/${asset_name}.tar.gz" "$download_url"

    tar -xzf "${tmpdir}/${asset_name}.tar.gz" -C "$tmpdir"
    install_binary "${tmpdir}/${BINARY}" "$version"
}

# ── Install from source (fallback) ─────────────────────────────────
install_from_source() {
    local version="${1:-master}"

    if ! command -v cargo &>/dev/null; then
        error "Neither release binary nor 'cargo' found. Please install Rust: https://rustup.rs"
        exit 1
    fi

    local tmpdir
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    info "Cloning repository (${version})..."
    git clone --depth 1 -b "$version" "${GITHUB}/${REPO}.git" "$tmpdir/src"

    info "Building release binary..."
    (cd "$tmpdir/src" && cargo build --release)

    install_binary "$tmpdir/src/target/release/${BINARY}" "source (${version})"
}

# ── Install binary ──────────────────────────────────────────────────
install_binary() {
    local src="$1"
    local source_desc="$2"
    local dest

    # Determine install location
    if [ -w "/usr/local/bin" ]; then
        dest="/usr/local/bin"
    elif command -v sudo &>/dev/null; then
        dest="/usr/local/bin"
        info "Installing to /usr/local/bin (requires sudo)"
        sudo install -m 755 "$src" "${dest}/${BINARY}"
        info "Installed ${BINARY} ${source_desc} to ${dest}/${BINARY}"
        return
    else
        dest="${HOME}/.local/bin"
        mkdir -p "$dest"
    fi

    install -m 755 "$src" "${dest}/${BINARY}"
    info "Installed ${BINARY} ${source_desc} to ${dest}/${BINARY}"
}

# ── Main ───────────────────────────────────────────────────────────
main() {
    local version="${1:-latest}"

    info "Installing ${BINARY} (${version})..."
    check_deps

    local target
    target=$(detect_target)
    info "Target: ${target}"

    # Try release first, fall back to source build
    if ! install_from_release "$target" "$version"; then
        warn "No release binary found for ${target}, building from source..."
        if [ "$version" = "latest" ]; then
            install_from_source "master"
        else
            install_from_source "$version"
        fi
    fi

    # Verify installation
    if command -v "$BINARY" &>/dev/null; then
        info "Success! Run '${BINARY}' to get started."
    else
        warn "${BINARY} may not be in your PATH."
        warn "Add the install location to PATH or run: /usr/local/bin/${BINARY}"
    fi
}

main "$@"
