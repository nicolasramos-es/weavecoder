# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Cleaned up stale internal comments and renamed an internal launcher variable for consistency with the product identity (NRA-762).

## [0.68.2] — 2026-08-25

### Added

- Multiple OpenAI-compatible providers from the TUI: choosing "OpenAI-compatible" now walks you through name → base URL → model → API key, and each provider is saved as a named `[providers.<name>]` profile so you can add as many as you need (NRA-763).
- Local endpoint providers (Ollama, LM Studio, llama.cpp, vLLM, oMLX) now let you edit the base URL before the API-key step, persisting the override so you can point a local provider at a different machine or port (NRA-763).
- Permission selector: `[tools] disk_mode` (`full` / `limited` / `ask`) plus per-tool `allow` / `ask` / `deny` overrides, manageable from the new `/permissions` TUI command. Tools flagged `ask` queue an approval request; `deny` blocks them (NRA-763).
- Removed the legacy Weavecoder subscription/login flow, which had no backend. The login picker no longer offers a subscription provider.

### Fixed

- Pasting an API key or endpoint during a login could swallow the trailing Enter, leaving the login stuck. A pasted value is now inserted verbatim during a pending login (NRA-763).

## [0.68.1] — 2026-08-24

### Changed

- Removed the legacy `solosystems.dev` discovery/account endpoints from the allowlists. The Weavecoder API and account URLs are the only accepted endpoints going forward; configs pointing at the deprecated host are treated as hand-written choices.
- Product-identity and branding cleanup (NRA-762): onboarding, telemetry and launcher messaging are fully aligned with Weavecoder. The MIT LICENSE attribution to the original author is preserved as required.

## [0.68.0] — 2026-08-24

_Not released publicly (superseded by 0.68.1)._

## [0.67.0] — 2026-08-16

Current development version (`Cargo.toml`). This is the first entry of the
project's own changelog, so it aggregates the changes present on `main`.

### Added

- **One-line installers** — `scripts/install.sh` (Linux/macOS) and
  `scripts/install.ps1` (Windows) that download the release binary from GitHub
  Releases and verify its SHA-256 checksum against the published digest
  (`WVC_RELEASE_METADATA_BASE`, `valid_release_tag`).
- **Code Knowledge Graph (CKG)** — `wvc init <project> [--db <path>]` and
  `wvc code-search <query> [--db <path>] [--top-k N]` backed by tree-sitter
  parsing (Go, Python/`.pyi`, TypeScript/TSX, Rust), SQLite + FTS5 storage
  (`crates/wvc-code-graph`), incremental indexing with SHA-256 snapshots, and
  hybrid search fused with Reciprocal Rank Fusion.
- **Local embeddings** — `crates/wvc-embedding`: all-MiniLM-L6-v2 (384 dims)
  in ONNX with `tract-onnx` inference and a HuggingFace tokenizer, downloaded
  on demand.
- **Agent Swarm** — native orchestration of multiple parallel requests via
  `crates/wvc-swarm-core`.
- **Provider support** — Ollama, LM Studio, oMLX/llama.cpp and any
  OpenAI-compatible endpoint, plus cloud providers (full list in `wvc --help`).
- **User documentation** — `docs/installation.md`, `docs/commands.md`,
  `docs/architecture.md` and `docs/troubleshooting.md`.
- **Complete main README** in English, with the Weavecoder logo in the header
  and the final app icon (`Weavecoder.icns`, `assets/app-icons`).
- **Meta-documents** — `CHANGELOG.md`, `CONTRIBUTING.md` and `SECURITY.md` at
  the repository root.

### Changed

- Install domain updated to `weavecoder.nramos.dev` and the README unified to
  English. The README install commands point to
  `raw.githubusercontent.com`; the installer's metadata fallback
  (`RELEASE_METADATA_BASE` in `scripts/install.sh`) still defaults to a legacy
  web domain and is tracked in NRA-528.
- CI: all Linux jobs run on the self-hosted runner; macOS/Windows workflows are
  manual-only (GitHub-hosted) to control costs; CI runs on pull requests only.
- Clippy `-D warnings` scoped to `wvc-code-graph` (Phase 1 crate); the
  workspace-wide lint is informational.
- `Cargo.lock` committed to the repository.

### Fixed

- Clippy warnings across `wvc-sdk`, `wvc-tui`, `wvc-math`, `wvc-app-core`,
  `wvc-core`, `wvc-base`, `wvc-setup-hints`, `terminal-launch` and
  `wvc-code-graph` (including clippy 1.97 `-D warnings`).
- E2E system prompt identity test now accepts Weavecoder.
- Fork author references cleaned from the codebase (maintainer, URLs, LICENSE).

[Unreleased]: https://github.com/nicolasramos-es/weavecoder/compare/v0.67.0-dev...HEAD
[0.67.0]: https://github.com/nicolasramos-es/weavecoder/releases/tag/v0.67.0-dev
