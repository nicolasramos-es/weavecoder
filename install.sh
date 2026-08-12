#!/usr/bin/env bash
# Weavecoder installer — https://weavecoder.sh/install
# Downloads the latest wvc binary from GitHub Releases.
set -euo pipefail

REPO="nicolasramos/weavecoder"
INSTALL_DIR="${WVC_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${WVC_VERSION:-latest}"

info() { printf '\033[1;34m%s\033[0m\n' "$*"; }
err()  { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

# --- Pre-flight: curl is required ---
command -v curl >/dev/null 2>&1 || err "curl is required but not found in PATH"

# --- GitHub auth helper for private repos ---
GITHUB_TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"

gh_curl() {
  # Usage: gh_curl <url> [output_file]
  # Wraps curl with GitHub auth when a token is available.
  local url="$1"
  local outfile="${2:-}"
  if [ -n "$GITHUB_TOKEN" ]; then
    curl -fsSL -H "Authorization: token ${GITHUB_TOKEN}" "$url" ${outfile:+-o "$outfile"}
  else
    curl -fsSL "$url" ${outfile:+-o "$outfile"}
  fi
}

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
  VERSION="$(gh_curl "https://api.github.com/repos/${REPO}/releases/latest" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)"
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
gh_curl "$URL" "$TMP" || err "Download failed: $URL"
chmod +x "$TMP"

# --- SHA-256 checksum verification ---
# Fetch the release JSON and extract the digest for this asset.
RELEASE_JSON="$(gh_curl "https://api.github.com/repos/${REPO}/releases/tags/${VERSION}")"
EXPECTED_DIGEST="$(printf '%s' "$RELEASE_JSON" | grep -o '"digest": *"[^"]*"' | head -1 || true)"

if [ -z "$EXPECTED_DIGEST" ]; then
  err "Could not obtain SHA-256 digest for $ASSET from release ${VERSION}; aborting for safety"
fi

# Strip the "sha256:" prefix to get the raw hex.
EXPECTED_HASH="${EXPECTED_DIGEST#sha256:}"

# Compute local hash (portable across macOS / Linux).
if command -v shasum >/dev/null 2>&1; then
  ACTUAL_HASH="$(shasum -a 256 "$TMP" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_HASH="$(sha256sum "$TMP" | awk '{print $1}')"
elif command -v openssl >/dev/null 2>&1; then
  ACTUAL_HASH="$(openssl dgst -sha256 "$TMP" | awk '{print $NF}')"
else
  err "No SHA-256 tool available (need shasum, sha256sum, or openssl)"
fi

if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
  err "Checksum mismatch for $ASSET — expected ${EXPECTED_HASH}, got ${ACTUAL_HASH}"
fi

info "SHA-256 verified: $ASSET"

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
