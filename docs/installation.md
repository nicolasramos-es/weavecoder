# Installation

Install **wvc** (Weavecoder CLI) on macOS, Linux, or Windows 11+.

## Quick Install (Recommended)

The installer fetches the latest release binary, verifies its SHA-256 checksum against the digest published in the GitHub Release, and aborts with a clear error if verification fails.

### macOS & Linux

```bash
curl -fsSL https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.sh | bash
```

### Windows 11 (PowerShell 5.1+)

```powershell
irm https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.ps1 | iex
```

> The installer verifies the **SHA-256 checksum** of the binary against the digest published in the release and aborts with a clear error if they don't match.

### Private Repositories

If the repository is private, export a token before running the installer:

```bash
export WVC_GITHUB_TOKEN=ghp_your_token_here   # or: export GITHUB_TOKEN=...
```

The installer checks both `WVC_GITHUB_TOKEN` and `GITHUB_TOKEN`.

## Build from Source

Requires a recent stable Rust toolchain (edition 2024 or later).

```bash
git clone https://github.com/nicolasramos-es/weavecoder.git
cd weavecoder
cargo build --release --bin wvc
# → target/release/wvc
```

## Verify Installation

After installation, verify the binary works:

```bash
wvc --version
# or equivalently:
wvc version
```

## Uninstall

### macOS & Linux

```bash
curl -fsSL https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/scripts/uninstall.sh | bash
```

With options:

| Flag      | Effect                                          |
|-----------|---------------------------------------------------|
| `--purge`  | Also deletes user data (`~/.wvc`)                 |
| `--dry-run` | Print what would be removed without deleting      |
| `--yes`    | Skip the confirmation prompt                      |

Example: `bash scripts/uninstall.sh --purge`

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/scripts/uninstall.ps1 | iex
```

Parameters: `-Purge` (also delete user data), `-DryRun`, `-Yes`.

## Troubleshooting

For common issues and their solutions, see [docs/troubleshooting.md](./troubleshooting.md).
