#!/usr/bin/env bash
# Weavecoder installer — https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.sh
# Downloads the latest wvc binary from GitHub Releases and verifies its SHA-256 checksum.
set -euo pipefail

REPO="nicolasramos-es/weavecoder"
INSTALL_DIR="${WVC_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${WVC_VERSION:-latest}"
# Optional auth for private repositories (the product repo is private until launch).
GITHUB_TOKEN="${WVC_GITHUB_TOKEN:-${GITHUB_TOKEN:-}}"

info() { printf '\033[1;34m%s\033[0m\n' "$*"; }
err()  { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

# curl is required
command -v curl >/dev/null 2>&1 || err "curl is required to install wvc. Install curl and re-run this script."

# sha256_file: portable SHA-256 of a file (sha256sum | shasum | openssl)
sha256_file() {
  file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print tolower($1)}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print tolower($1)}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$file" | awk '{print tolower($NF)}'
  else
    return 1
  fi
}

# Detect OS + arch and map to the release asset naming convention.
# Assets on GitHub Releases:
#   macOS arm64    -> wvc-macos-aarch64.tar.gz
#   macOS x86_64   -> wvc-macos-x86_64.tar.gz
#   Linux x86_64   -> wvc-linux-x86_64
#   Linux aarch64  -> wvc-linux-aarch64
#   Windows x86_64 -> wvc-windows-x86_64.exe
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_NAME="macos" ;;
  Linux)  OS_NAME="linux" ;;
  MINGW*|MSYS*|CYGWIN*) OS_NAME="windows" ;;
  *) err "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
  arm64|aarch64) ARCH_NAME="aarch64" ;;
  x86_64|amd64)  ARCH_NAME="x86_64" ;;
  *) err "Unsupported architecture: $ARCH" ;;
esac

ASSET="wvc-${OS_NAME}-${ARCH_NAME}"
if [ "$OS_NAME" = "macos" ]; then
  ASSET_FILENAME="${ASSET}.tar.gz"
else
  [ "$OS_NAME" = "windows" ] && ASSET_FILENAME="${ASSET}.exe" || ASSET_FILENAME="${ASSET}"
fi

# Base curl args; add auth header when a token is provided (private repo)
CURL_ARGS=(-fsSL)
if [ -n "$GITHUB_TOKEN" ]; then
  CURL_ARGS+=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
fi

# Resolve the latest release tag (or honor a pinned WVC_VERSION).
if [ "$VERSION" = "latest" ]; then
  VERSION="$(curl "${CURL_ARGS[@]}" "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4)"
  [ -n "$VERSION" ] || err "Could not resolve latest release from GitHub"
fi

GITHUB_RELEASE_BASE="https://github.com/${REPO}/releases/download/${VERSION}"

info "Weavecoder installer"
info "  OS:      ${OS_NAME} (${ARCH})"
info "  Version: ${VERSION}"
info "  Asset:   ${ASSET_FILENAME}"

mkdir -p "$INSTALL_DIR"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

# Fetch the published SHA-256 manifest and the asset.
info "Downloading ${ASSET_FILENAME}..."
CHECKSUMS="$(curl "${CURL_ARGS[@]}" "$GITHUB_RELEASE_BASE/SHA256SUMS" || true)"
curl "${CURL_ARGS[@]}" "$GITHUB_RELEASE_BASE/${ASSET_FILENAME}" -o "$TMPDIR/download" \
  || err "Download failed: $GITHUB_RELEASE_BASE/${ASSET_FILENAME}"

# Verify SHA-256 against the manifest (format: "<hex>  <asset-name>").
EXPECTED="$(printf '%s' "$CHECKSUMS" \
  | awk -v a="$ASSET_FILENAME" '$2 == a || $2 == "*" a { print tolower($1); exit }')"
if ! printf '%s' "$EXPECTED" | grep -Eq '^[0-9a-f]{64}$'; then
  err "Could not find a trusted SHA-256 checksum for ${ASSET_FILENAME} in ${VERSION}/SHA256SUMS"
fi
ACTUAL="$(sha256_file "$TMPDIR/download")" \
  || err "sha256sum, shasum, or openssl is required to verify the download checksum"
[ "$ACTUAL" = "$EXPECTED" ] \
  || err "Checksum mismatch for ${ASSET_FILENAME} (${VERSION}); the download may be corrupted. Aborting."
info "Checksum OK."

# macOS ships a tarball; extract it. Others are raw binaries.
if [ "$OS_NAME" = "macos" ]; then
  tar xzf "$TMPDIR/download" -C "$TMPDIR"
  BIN_PATH="$TMPDIR/${ASSET}"
else
  BIN_PATH="$TMPDIR/download"
fi
chmod +x "$BIN_PATH"

# Verify it is the real binary before installing.
if ! "$BIN_PATH" --version >/dev/null 2>&1; then
  err "Downloaded file is not a valid wvc binary"
fi

mv "$BIN_PATH" "$INSTALL_DIR/wvc"

# macOS quarantine
if [ "$OS_NAME" = "macos" ]; then
  xattr -d com.apple.quarantine "$INSTALL_DIR/wvc" 2>/dev/null || true
fi

info "Installed wvc to $INSTALL_DIR/wvc"
info ""

# PATH hint
case ":$PATH:" in
  *":$INSTALL_DIR:") ;;
  *) info "Add to your PATH:  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac

info "Verify with:  wvc --version"
"$INSTALL_DIR/wvc" --version
