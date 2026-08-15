#!/usr/bin/env bash
# Weavecoder installer — https://weavecoder.sh/install
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

# Base curl args; add auth header when a token is provided (private repo)
CURL_ARGS=(-fsSL)
if [ -n "$GITHUB_TOKEN" ]; then
  CURL_ARGS+=(-H "Authorization: Bearer $GITHUB_TOKEN")
fi

# Fetch release metadata (latest or pinned tag) — the same response carries
# the tag name, the asset id, and the published SHA-256 digest.
if [ "$VERSION" = "latest" ]; then
  RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
else
  RELEASE_URL="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
fi

RELEASE_JSON="$(curl "${CURL_ARGS[@]}" "$RELEASE_URL")" \
  || err "Could not fetch release metadata from $RELEASE_URL (is the release public, or is a GITHUB_TOKEN required?)"

if [ "$VERSION" = "latest" ]; then
  VERSION="$(printf '%s' "$RELEASE_JSON" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)"
  [ -n "$VERSION" ] || err "Could not resolve latest release"
fi

# Extract the published SHA-256 digest for this asset (assets[].digest, "sha256:<hex>")
ASSET_DIGEST="$(printf '%s' "$RELEASE_JSON" \
  | grep -A60 '"name": *"'"$ASSET"'"' \
  | grep -o '"digest": *"sha256:[0-9a-f]\{64\}"' | head -1 | sed 's/.*sha256://; s/"//')"
# Fallback for minified (single-line) API responses
if [ -z "$ASSET_DIGEST" ]; then
  ASSET_DIGEST="$(printf '%s' "$RELEASE_JSON" \
    | grep -o '"name": *"'"$ASSET"'".\{0,2000\}"digest": *"sha256:[0-9a-f]\{64\}"' \
    | grep -o 'sha256:[0-9a-f]\{64\}' | head -1)"
fi
[ -n "$ASSET_DIGEST" ] || err "Could not find a SHA-256 digest for ${ASSET} in release ${VERSION}; cannot verify the download."

# Asset id — needed to download from a private repo via the API asset endpoint
ASSET_ID="$(printf '%s' "$RELEASE_JSON" \
  | grep -B10 '"name": *"'"$ASSET"'"' \
  | grep -o '"id": *[0-9][0-9]*' | tail -1 | grep -o '[0-9][0-9]*')"

# Private repos require the API asset endpoint; public repos use the direct URL
if [ -n "$GITHUB_TOKEN" ]; then
  URL="https://api.github.com/repos/${REPO}/releases/assets/${ASSET_ID}"
  DOWNLOAD_ARGS=(-fsSL -H "Authorization: Bearer $GITHUB_TOKEN" -H "Accept: application/octet-stream")
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
  DOWNLOAD_ARGS=(-fsSL)
fi

info "Weavecoder installer"
info "  OS:     ${OS_NAME} (${ARCH})"
info "  Version: ${VERSION}"
info "  URL:    ${URL}"

mkdir -p "$INSTALL_DIR"

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

info "Downloading..."
curl "${DOWNLOAD_ARGS[@]}" "$URL" -o "$TMP" || err "Download failed: $URL"

# Verify SHA-256 checksum against the digest published with the release
info "Verifying SHA-256 checksum..."
ACTUAL_SHA256="$(sha256_file "$TMP")" \
  || err "sha256sum, shasum, or openssl is required to verify the download checksum"
if [ "$ACTUAL_SHA256" != "$ASSET_DIGEST" ]; then
  err "Checksum mismatch for ${ASSET} (${VERSION}): expected ${ASSET_DIGEST}, got ${ACTUAL_SHA256}. The downloaded file may be corrupted or tampered with; aborting."
fi
info "Checksum OK (${ASSET_DIGEST})"
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
