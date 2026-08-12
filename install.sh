#!/usr/bin/env bash
# Weavecoder installer — https://weavecoder.sh/install
# Downloads the latest wvc binary from GitHub Releases.
set -euo pipefail

REPO="nicolasramos/weavecoder"
INSTALL_DIR="${WVC_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${WVC_VERSION:-latest}"

info() { printf '\033[1;34m%s\033[0m\n' "$*"; }
err()  { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

# Detect OS + arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_NAME="macos" ;;
  Linux)  OS_NAME="linux" ;;
  *) err "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
  arm64|aarch64) ARCH_NAME="arm64" ;;
  x86_64|amd64)  ARCH_NAME="x86_64" ;;
  *) err "Unsupported architecture: $ARCH" ;;
esac

ASSET="wvc-${OS_NAME}-${ARCH_NAME}"

# Resolve latest release tag if requested
if [ "$VERSION" = "latest" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)"
  [ -n "$VERSION" ] || err "Could not resolve latest release"
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

info "Weavecoder installer"
info "  OS:     ${OS_NAME} (${ARCH})"
info "  Version: ${VERSION}"
info "  URL:    ${URL}"

mkdir -p "$INSTALL_DIR"

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

info "Downloading..."
curl -fsSL "$URL" -o "$TMP" || err "Download failed: $URL"
chmod +x "$TMP"

# Verify it is the real binary before installing
if ! "$TMP" --version >/dev/null 2>&1; then
  err "Downloaded file is not a valid wvc binary"
fi

mv "$TMP" "$INSTALL_DIR/wvc"
rm -f "$TMP"

# macOS quarantine
if [ "$OS_NAME" = "macos" ]; then
  xattr -d com.apple.quarantine "$INSTALL_DIR/wvc" 2>/dev/null || true
fi

info "Installed wvc to $INSTALL_DIR/wvc"
info ""

# PATH hint
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) info "Add to your PATH:  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac

info "Verify with:  wvc --version"
"$INSTALL_DIR/wvc" --version
