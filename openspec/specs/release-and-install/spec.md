# Release & Install

## Purpose

Make installing and updating Weavecoder safe and reliable, with the GitHub
repository as the only source.

## Requirements

- All code, installers, updates, and downloads come **only** from the GitHub
  repository (`nicolamosramos-es/weavecoder`) — never from a web domain.
- The root `install.sh` maps OS/arch to the real release asset names
  (`wvc-macos-aarch64.tar.gz`, `wvc-macos-x86_64.tar.gz`, `wvc-linux-x86_64`,
  `wvc-windows-x86_64.exe`) and verifies the SHA-256 against the published
  `SHA256SUMS`.
- On macOS it extracts the tarball and strips the quarantine flag.
- Releases are built for 4 platforms from the same commit and published to
  GitHub Releases with a `SHA256SUMS` manifest.
- `wvc update` / installer resolves the latest release; a pinned version is
  supported via `WVC_VERSION`.
- Every release updates `docs/` and the README — documentation is updated with
  every change (non-negotiable).
